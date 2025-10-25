// jkconfig Web界面JavaScript功能

// DOM元素
const statusContent = document.getElementById('status-content');

// 测试API端点
async function testApi(endpoint) {
    console.log(`Testing API endpoint: ${endpoint}`);

    // 显示加载状态
    statusContent.innerHTML = '<div style="color: #2563eb;">⏳ 正在请求...</div>';
    statusContent.className = 'status-success';

    try {
        const response = await fetch(endpoint);
        const data = await response.json();

        // 成功响应
        statusContent.innerHTML = `✅ API请求成功！

端点: ${endpoint}
状态码: ${response.status}

响应数据:
${JSON.stringify(data, null, 2)}

请求时间: ${new Date().toLocaleString()}`;
        statusContent.className = 'status-success';

        console.log('API Response:', data);

    } catch (error) {
        // 错误处理
        statusContent.innerHTML = `❌ API请求失败！

端点: ${endpoint}

错误信息:
${error.message}

请求时间: ${new Date().toLocaleString()}

请检查：
1. 服务器是否正在运行
2. 端点是否正确
3. 网络连接是否正常`;
        statusContent.className = 'status-error';

        console.error('API Error:', error);
    }
}

// 页面加载完成后的初始化
document.addEventListener('DOMContentLoaded', function() {
    console.log('🚀 jkconfig Web界面已加载');

    // 添加页面可见性变化监听
    document.addEventListener('visibilitychange', function() {
        if (!document.hidden) {
            console.log('📱 页面变为可见状态');
        }
    });

    // 添加键盘快捷键
    document.addEventListener('keydown', function(event) {
        // Ctrl+R 或 F5: 测试所有API
        if ((event.ctrlKey && event.key === 'r') || event.key === 'F5') {
            event.preventDefault();
            testAllApis();
        }
    });

    console.log('⌨️ 键盘快捷键: Ctrl+R 或 F5 测试所有API端点');
});

// 测试所有API端点
async function testAllApis() {
    console.log('🔄 开始测试所有API端点...');

    const endpoints = ['/api/config', '/api/health'];
    let results = [];

    for (const endpoint of endpoints) {
        try {
            const response = await fetch(endpoint);
            const data = await response.json();
            results.push({
                endpoint,
                status: response.status,
                success: true,
                data
            });
        } catch (error) {
            results.push({
                endpoint,
                success: false,
                error: error.message
            });
        }
    }

    // 显示汇总结果
    const successCount = results.filter(r => r.success).length;
    const totalCount = results.length;

    statusContent.innerHTML = `📊 API端点测试完成！

测试时间: ${new Date().toLocaleString()}
成功率: ${successCount}/${totalCount}

详细结果:
${results.map(r => {
    if (r.success) {
        return `✅ ${r.endpoint} (${r.status})`;
    } else {
        return `❌ ${r.endpoint} - ${r.error}`;
    }
}).join('\n')}`;

    statusContent.className = successCount === totalCount ? 'status-success' : 'status-error';

    console.log('📊 API测试结果:', results);
}

// 实用工具函数
const utils = {
    // 格式化JSON
    formatJson: function(obj) {
        return JSON.stringify(obj, null, 2);
    },

    // 获取当前时间戳
    timestamp: function() {
        return new Date().toISOString();
    },

    // 复制到剪贴板
    copyToClipboard: async function(text) {
        try {
            await navigator.clipboard.writeText(text);
            console.log('✅ 已复制到剪贴板');
        } catch (error) {
            console.error('❌ 复制失败:', error);
        }
    }
};

// 导出到全局作用域（用于调试）
window.jkconfig = {
    testApi,
    testAllApis,
    utils
};