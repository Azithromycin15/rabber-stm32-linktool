/**
 * ST-Link V2 Component Definition
 *
 * This JavaScript module defines the ST-Link V2 component metadata and interface.
 * It serves as a descriptor for the component's capabilities and is used by the
 * plugin system to understand what actions the component supports.
 */

export const STLinkV2Component = {
  // Component identification
  id: "stlink_v2",
  name: "ST-Link V2",
  componentType: "debugger",
  description: "ST-Link V2 downloader component for STM32 SWD flashing and MCU information.",

  // Module paths
  pythonModule: "plugins/stlink_v2/python/downloader.py",
  jsModule: "plugins/stlink_v2/js/component.js",

  // Hardware metadata
  metadata: {
    vendorId: "0x0483",
    productIds: ["0x3748", "0x374B"],
    supportedPlatforms: ["linux", "windows"],
    flashStartAddress: "0x08000000"
  },

  // Supported actions
  actions: ["probe", "info", "flash", "reset"],

  /**
   * Run method (placeholder for JS execution)
   *
   * This method is currently a placeholder. The actual execution is handled
   * by the Go plugin loader which calls the Python implementation.
   *
   * @param {string} action - The action to perform
   * @param {string} filePath - File path for flash action (optional)
   * @returns {object} Result object with action details
   */
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
