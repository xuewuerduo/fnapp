const API = '/api';

document.addEventListener('DOMContentLoaded', loadDevices);

document.getElementById('addModal').addEventListener('click', function(e) {
    if (e.target === this) closeAddModal();
});
document.getElementById('scanModal').addEventListener('click', function(e) {
    if (e.target === this) closeScanModal();
});

async function loadDevices() {
    try {
        const res = await fetch(`${API}/devices`);
        const devices = await res.json();
        renderDevices(devices);
    } catch (e) {
        showToast('加载设备列表失败', 'error');
    }
}

function renderDevices(devices) {
    const list = document.getElementById('deviceList');
    const count = document.getElementById('deviceCount');
    count.textContent = devices.length;

    if (devices.length === 0) {
        list.innerHTML = `
            <div class="empty-state">
                <div class="empty-icon">📡</div>
                <p>暂无常用设备</p>
                <p class="hint">扫描局域网或手动添加设备</p>
            </div>
        `;
        return;
    }

    list.innerHTML = devices.map(function(d) {
        return `
        <div class="device-card" data-mac="${escapeAttr(d.mac)}">
            <div class="device-header">
                <div class="device-name" onclick="editName('${escapeAttr(d.mac)}', this)">${escapeHtml(d.name || '未命名')}</div>
                ${d.vendor ? vendorBadge(d.vendor) : ''}
            </div>
            <div class="device-info">
                <div class="device-info-row">
                    <span class="device-info-label">IP</span>
                    <span>${escapeHtml(d.ip || '未知')}</span>
                </div>
                <div class="device-info-row">
                    <span class="device-info-label">MAC</span>
                    <span>${escapeHtml(d.mac)}</span>
                </div>
            </div>
            <div class="device-actions">
                <button class="btn-wake" onclick="wakeDevice('${escapeAttr(d.mac)}', this)">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg>
                    <span>唤醒</span>
                </button>
                <button class="btn-icon" onclick="editName('${escapeAttr(d.mac)}', this)" title="编辑备注">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
                </button>
                <button class="btn-icon danger" onclick="deleteDevice('${escapeAttr(d.mac)}')" title="删除设备">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                </button>
            </div>
        </div>
        `;
    }).join('');

    // 异步补充厂商信息
    devices.forEach(function(d) {
        if (!d.vendor) {
            fetchVendorAsync(d.mac, function(vendor) {
                if (!vendor) return;
                var cards = document.querySelectorAll('#deviceList .device-card');
                for (var i = 0; i < cards.length; i++) {
                    if (cards[i].dataset.mac === d.mac) {
                        var h = cards[i].querySelector('.device-header');
                        if (h && !h.querySelector('.vendor-name')) {
                            h.insertAdjacentHTML('beforeend', vendorBadge(vendor));
                        }
                        break;
                    }
                }
            });
        }
    });
}

async function scanDevices() {
    const btn = document.getElementById('scanBtn');
    btn.disabled = true;

    try {
        const res = await fetch(`${API}/scan`, { method: 'POST' });
        const data = await res.json();

        if (res.ok) {
            showScanResults(data.devices || []);
        } else {
            showToast(data.error || '扫描失败', 'error');
        }
    } catch (e) {
        showToast('扫描失败，请检查网络', 'error');
    } finally {
        btn.disabled = false;
    }
}

function showScanResults(devices) {
    const modal = document.getElementById('scanModal');
    const list = document.getElementById('scanResultList');
    const count = document.getElementById('scanCount');

    if (devices.length === 0) {
        list.innerHTML = '<div class="empty-state"><p>未发现新设备</p></div>';
    } else {
        count.textContent = devices.length;
        list.innerHTML = devices.map(function(d) {
            const btnHtml = d.exists
                ? '<span class="scan-added">已添加</span>'
                : `<button class="btn-wake" onclick="addScannedDevice('${escapeAttr(d.mac)}', '${escapeAttr(d.ip)}', this)"><span>+ 添加</span></button>`;
            return `
            <div class="scan-device">
                <div class="scan-device-info">
                    <div class="scan-device-mac">${escapeHtml(d.mac)}</div>
                    <div class="scan-device-ip">${escapeHtml(d.ip)}</div>
                    ${d.vendor ? vendorBadge(d.vendor) : ''}
                </div>
                <div class="scan-device-action">${btnHtml}</div>
            </div>
            `;
        }).join('');
    }

    modal.style.display = 'flex';

    devices.forEach(function(d) {
        if (!d.vendor) {
            fetchVendorAsync(d.mac, function(vendor) {
                var el = document.querySelector('#scanResultList .scan-device-mac');
                if (el) {
                    var rows = document.querySelectorAll('#scanResultList .scan-device');
                    for (var i = 0; i < rows.length; i++) {
                        if (rows[i].querySelector('.scan-device-mac').textContent === d.mac) {
                            var info = rows[i].querySelector('.scan-device-info');
                            if (vendor) {
                                info.insertAdjacentHTML('beforeend', vendorBadge(vendor));
                            }
                            break;
                        }
                    }
                }
            });
        }
    });
}

function fetchVendorAsync(mac, callback) {
    fetch('/api/vendor?mac=' + encodeURIComponent(mac))
        .then(function(r) { return r.json(); })
        .then(function(data) {
            if (!callback) return;
            var v = data.vendor;
            if (v && !v.startsWith('{') && !v.startsWith('[') && v.length < 100) {
                callback(v);
            } else {
                callback(null);
            }
        })
        .catch(function() { if (callback) callback(null); });
}

function closeScanModal() {
    document.getElementById('scanModal').style.display = 'none';
}

async function addScannedDevice(mac, ip, btn) {
    btn.disabled = true;
    btn.innerHTML = '<span class="loading"></span>';

    try {
        const res = await fetch(`${API}/devices`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ mac: mac, name: mac, ip: ip })
        });

        if (res.ok) {
            showToast('设备已添加', 'success');
            btn.outerHTML = '<span class="scan-added">已添加</span>';
            loadDevices();
        } else {
            const data = await res.json();
            showToast(data.error || '添加失败', 'error');
            btn.disabled = false;
            btn.innerHTML = '+ 添加';
        }
    } catch (e) {
        showToast('添加失败', 'error');
        btn.disabled = false;
        btn.innerHTML = '+ 添加';
    }
}

async function wakeDevice(mac, btn) {
    const original = btn.innerHTML;
    btn.innerHTML = '<span class="loading"></span>';
    btn.disabled = true;

    try {
        const res = await fetch(`${API}/devices/${encodeURIComponent(mac)}/wake`, { method: 'POST' });
        const data = await res.json();

        if (res.ok) {
            showToast('唤醒包已发送', 'success');
        } else {
            showToast(data.error || '唤醒失败', 'error');
        }
    } catch (e) {
        showToast('唤醒失败', 'error');
    } finally {
        btn.innerHTML = original;
        btn.disabled = false;
    }
}

function showAddModal() {
    document.getElementById('addModal').style.display = 'flex';
    document.getElementById('macInput').value = '';
    document.getElementById('nameInput').value = '';
    document.getElementById('ipInput').value = '';
    setTimeout(function() { document.getElementById('macInput').focus(); }, 100);
}

function closeAddModal() {
    document.getElementById('addModal').style.display = 'none';
}

async function addDevice() {
    const mac = document.getElementById('macInput').value.trim();
    const name = document.getElementById('nameInput').value.trim();
    const ip = document.getElementById('ipInput').value.trim();

    if (!mac) {
        showToast('请输入 MAC 地址', 'error');
        return;
    }
    if (!name) {
        showToast('请输入备注名称', 'error');
        return;
    }

    try {
        const res = await fetch(`${API}/devices`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ mac: mac, name: name, ip: ip || null })
        });
        const data = await res.json();

        if (res.ok) {
            showToast('设备添加成功', 'success');
            closeAddModal();
            loadDevices();
        } else {
            showToast(data.error || '添加失败', 'error');
        }
    } catch (e) {
        showToast('添加失败', 'error');
    }
}

async function deleteDevice(mac) {
    if (!confirm('确定删除此设备？')) return;

    try {
        const res = await fetch(`${API}/devices/${encodeURIComponent(mac)}`, { method: 'DELETE' });
        if (res.ok) {
            showToast('设备已删除', 'success');
            loadDevices();
        } else {
            showToast('删除失败', 'error');
        }
    } catch (e) {
        showToast('删除失败', 'error');
    }
}

function editName(mac, elem) {
    const card = elem.closest('.device-card');
    const nameDiv = card.querySelector('.device-name');
    if (!nameDiv || nameDiv.tagName === 'INPUT') return;

    const currentName = nameDiv.textContent === '未命名' ? '' : nameDiv.textContent;

    const input = document.createElement('input');
    input.type = 'text';
    input.className = 'name-edit';
    input.value = currentName;
    input.placeholder = '输入备注名称';

    nameDiv.replaceWith(input);
    input.focus();
    input.select();

    let saved = false;
    async function save() {
        if (saved) return;
        saved = true;
        const newName = input.value.trim();
        if (newName === currentName) {
            loadDevices();
            return;
        }
        try {
            const res = await fetch(`${API}/devices/${encodeURIComponent(mac)}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ name: newName })
            });
            if (res.ok) {
                showToast('备注已更新', 'success');
            } else {
                showToast('更新失败', 'error');
            }
        } catch (e) {
            showToast('更新失败', 'error');
        }
        loadDevices();
    }

    input.addEventListener('blur', save);
    input.addEventListener('keydown', function(e) {
        if (e.key === 'Enter') input.blur();
        if (e.key === 'Escape') { saved = true; loadDevices(); }
    });
}

function showToast(msg, type) {
    const toast = document.getElementById('toast');
    toast.textContent = msg;
    toast.className = 'toast show' + (type ? ' ' + type : '');
    clearTimeout(toast._timer);
    toast._timer = setTimeout(function() {
        toast.className = 'toast';
    }, 2500);
}

function vendorBadge(name) {
    var h = 0;
    for (var i = 0; i < name.length; i++) { h = name.charCodeAt(i) + ((h << 5) - h); }
    return '<span class="vendor-name" style="background:hsl(' + (h % 360) + ', 40%, 55%)">' + escapeHtml(name) + '</span>';
}

function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}

function escapeAttr(str) {
    return String(str).replace(/'/g, '&#39;').replace(/"/g, '&quot;');
}
