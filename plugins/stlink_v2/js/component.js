export const STLinkV2Component = {
  id: "stlink_v2",
  name: "ST-Link V2",
  componentType: "debugger",
  description: "ST-Link V2 downloader component for STM32 SWD flashing and MCU information.",
  pythonModule: "plugins/stlink_v2/python/downloader.py",
  jsModule: "plugins/stlink_v2/js/component.js",
  metadata: {
    vendorId: "0x0483",
    productIds: ["0x3748", "0x374B"],
    supportedPlatforms: ["linux", "windows"],
    flashStartAddress: "0x08000000"
  },
  actions: ["probe", "info", "flash", "reset"],
  run(action, filePath) {
    console.log(`组件 '${this.name}' 不直接在 JS 中执行。`);
    console.log(`请使用 Go 组件加载器调用 Python 实现。`);
    return {
      action,
      filePath,
      message: "JS 组件当前用于描述和元数据展示。"
    };
  }
};
