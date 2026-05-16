// Package main implements a plugin loader for the ST-Link tool.
// It loads plugin manifests and executes Python-based components.
package main

import (
    "encoding/json"
    "flag"
    "fmt"
    "io/ioutil"
    "os"
    "os/exec"
    "path/filepath"

    "gopkg.in/yaml.v3"
)

// ComponentMetadata holds hardware-specific information for a component.
type ComponentMetadata struct {
    VendorID           string   `yaml:"vendor_id" json:"vendor_id"`
    ProductIDs         []string `yaml:"product_ids" json:"product_ids"`
    SupportedPlatforms []string `yaml:"supported_platforms" json:"supported_platforms"`
    FlashStartAddress  string   `yaml:"flash_start_address" json:"flash_start_address"`
}

// ComponentAction defines an action that a component can perform.
type ComponentAction struct {
    Name        string `yaml:"name" json:"name"`
    Description string `yaml:"description" json:"description"`
    Args        string `yaml:"args,omitempty" json:"args,omitempty"`
}

// ComponentInfo contains all information about a plugin component.
type ComponentInfo struct {
    ID            string            `yaml:"id" json:"id"`
    PluginName    string            `yaml:"plugin_name" json:"plugin_name"`
    Command       string            `yaml:"command" json:"command"`
    Name          string            `yaml:"name" json:"name"`
    ComponentType string            `yaml:"component_type" json:"component_type"`
    Description   string            `yaml:"description" json:"description"`
    PythonModule  string            `yaml:"python_module" json:"python_module"`
    JSModule      string            `yaml:"js_module" json:"js_module"`
    Metadata      ComponentMetadata `yaml:"metadata" json:"metadata"`
    Actions       []ComponentAction `yaml:"actions" json:"actions"`
}

// PluginManifest represents the root structure of the plugin manifest YAML.
type PluginManifest struct {
    Components []ComponentInfo `yaml:"components"`
}

// loadManifest reads and parses the plugin manifest YAML file.
func loadManifest(path string) (*PluginManifest, error) {
    content, err := ioutil.ReadFile(path)
    if err != nil {
        return nil, err
    }
    var manifest PluginManifest
    if err := yaml.Unmarshal(content, &manifest); err != nil {
        return nil, err
    }
    return &manifest, nil
}

// findComponent searches for a component by its ID in the manifest.
func findComponent(manifest *PluginManifest, id string) *ComponentInfo {
    for _, component := range manifest.Components {
        if component.ID == id {
            return &component
        }
    }
    return nil
}

func dirExists(path string) bool {
    info, err := os.Stat(path)
    return err == nil && info.IsDir()
}

func pluginRoot() string {
    if envRoot := os.Getenv("PLUGIN_ROOT"); envRoot != "" {
        return envRoot
    }

    cwd, err := os.Getwd()
    if err == nil {
        candidate := filepath.Join(cwd, "plugins")
        if dirExists(candidate) {
            return candidate
        }
        candidate = filepath.Join(cwd, "..", "plugins")
        if dirExists(candidate) {
            return candidate
        }
    }

    return "plugins"
}

func pluginRootFromManifest(manifestPath string) string {
    abs, err := filepath.Abs(manifestPath)
    if err == nil {
        parent := filepath.Dir(abs)
        if filepath.Base(parent) == "plugins" && dirExists(parent) {
            return filepath.Dir(parent)
        }
        if dirExists(parent) {
            return parent
        }
    }
    return pluginRoot()
}

func saveManifest(path string, manifest *PluginManifest) error {
    if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
        return err
    }
    data, err := yaml.Marshal(manifest)
    if err != nil {
        return err
    }
    return os.WriteFile(path, data, 0o644)
}

func scanPlugins(root string) (*PluginManifest, error) {
    files, err := os.ReadDir(root)
    if err != nil {
        return nil, err
    }

    manifest := &PluginManifest{}
    for _, file := range files {
        if !file.IsDir() {
            continue
        }
        componentJSON := filepath.Join(root, file.Name(), "js", "component.json")
        data, err := os.ReadFile(componentJSON)
        if err != nil {
            continue
        }
        var component ComponentInfo
        if err := json.Unmarshal(data, &component); err != nil {
            continue
        }
        manifest.Components = append(manifest.Components, component)
    }

    if len(manifest.Components) == 0 {
        return nil, fmt.Errorf("no plugin components found in %s", root)
    }
    return manifest, nil
}

// listComponents prints all available components in the manifest.
func listComponents(manifest *PluginManifest) {
    fmt.Println("Available components:")
    for _, component := range manifest.Components {
        fmt.Printf("- %s (%s): %s\n", component.Name, component.ID, component.Description)
    }
}

// executePython runs the Python script for the specified component and action.
func executePython(component *ComponentInfo, action string, filePath string, address string, noVerify bool, extraArgs []string, pluginRoot string) error {
    scriptPath := filepath.Clean(component.PythonModule)
    if !filepath.IsAbs(scriptPath) {
        // If relative, treat it as relative to the repository root or plugin root.
        scriptPath = filepath.Join(pluginRoot, scriptPath)
    }
    args := []string{scriptPath, "--action", action}
    if filePath != "" {
        args = append(args, "--file", filePath)
    }
    if address != "" {
        args = append(args, "--address", address)
    }
    if noVerify {
        args = append(args, "--no-verify")
    }
	args = append(args, extraArgs...)
    cmd := exec.Command("python3", args...)
    cmd.Stdout = os.Stdout
    cmd.Stderr = os.Stderr
    return cmd.Run()
}

// main is the entry point of the plugin loader.
func main() {
    manifestPath := flag.String("manifest", "plugins/manifest.yaml", "Path to plugin manifest YAML")
    list := flag.Bool("list", false, "List available plugin components")
    componentID := flag.String("component", "stlink_v2", "Component ID to load")
    action := flag.String("action", "info", "Action to execute: probe, info, flash, reset, verify")
    filePath := flag.String("file", "", "File path for flash/verify action")
    address := flag.String("address", "", "Start address for flash/verify action")
    noVerify := flag.Bool("no-verify", false, "Skip verification after flash")
    flag.Parse()

    pluginRootPath := pluginRootFromManifest(*manifestPath)
    manifest, err := loadManifest(*manifestPath)
    if err != nil {
        pluginRootPath = pluginRoot()
        fmt.Fprintf(os.Stderr, "Warning: failed to load plugin manifest: %v\n", err)
        fmt.Fprintf(os.Stderr, "Attempting to scan plugins from %s...\n", pluginRootPath)
        manifest, err = scanPlugins(pluginRootPath)
        if err != nil {
            fmt.Fprintf(os.Stderr, "Failed to discover plugin components: %v\n", err)
            os.Exit(1)
        }
        if err := saveManifest(*manifestPath, manifest); err != nil {
            fmt.Fprintf(os.Stderr, "Warning: failed to save generated manifest: %v\n", err)
        }
    }

    if *list {
        listComponents(manifest)
        return
    }

    component := findComponent(manifest, *componentID)
    if component == nil {
        fmt.Fprintf(os.Stderr, "Component '%s' not found in manifest.\n", *componentID)
        os.Exit(1)
    }

    if *action == "flash" || *action == "verify" {
        if *filePath == "" {
            fmt.Fprintf(os.Stderr, "%s action requires --file <path>\n", *action)
            os.Exit(1)
        }
    }

    fmt.Printf("Loading component '%s' (%s)\n", component.Name, component.ID)
    err = executePython(component, *action, *filePath, *address, *noVerify, flag.Args(), pluginRootPath)
    if err != nil {
        fmt.Fprintf(os.Stderr, "Component execution failed: %v\n", err)
        os.Exit(1)
    }
}
