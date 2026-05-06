// Package main implements a plugin loader for the ST-Link tool.
// It loads plugin manifests and executes Python-based components.
package main

import (
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
    VendorID           string   `yaml:"vendor_id"`
    ProductIDs         []string `yaml:"product_ids"`
    SupportedPlatforms []string `yaml:"supported_platforms"`
    FlashStartAddress  string   `yaml:"flash_start_address"`
}

// ComponentAction defines an action that a component can perform.
type ComponentAction struct {
    Name        string `yaml:"name"`
    Description string `yaml:"description"`
    Args        string `yaml:"args,omitempty"`
}

// ComponentInfo contains all information about a plugin component.
type ComponentInfo struct {
    ID            string            `yaml:"id"`
    Name          string            `yaml:"name"`
    ComponentType string            `yaml:"component_type"`
    Description   string            `yaml:"description"`
    PythonModule  string            `yaml:"python_module"`
    JSModule      string            `yaml:"js_module"`
    Metadata      ComponentMetadata `yaml:"metadata"`
    Actions       []ComponentAction `yaml:"actions"`
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

// listComponents prints all available components in the manifest.
func listComponents(manifest *PluginManifest) {
    fmt.Println("Available components:")
    for _, component := range manifest.Components {
        fmt.Printf("- %s (%s): %s\n", component.Name, component.ID, component.Description)
    }
}

// executePython runs the Python script for the specified component and action.
func executePython(component *ComponentInfo, action string, filePath string) error {
    scriptPath := filepath.Clean(component.PythonModule)
    args := []string{scriptPath, "--action", action}
    if filePath != "" {
        args = append(args, "--file", filePath)
    }
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
    action := flag.String("action", "info", "Action to execute: probe, info, flash, reset")
    filePath := flag.String("file", "", "File path for flash action")
    flag.Parse()

    manifest, err := loadManifest(*manifestPath)
    if err != nil {
        fmt.Fprintf(os.Stderr, "Failed to load plugin manifest: %v\n", err)
        os.Exit(1)
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

    if *action == "flash" && *filePath == "" {
        fmt.Fprintf(os.Stderr, "flash action requires --file <path>\n")
        os.Exit(1)
    }

    fmt.Printf("Loading component '%s' (%s)\n", component.Name, component.ID)
    err = executePython(component, *action, *filePath)
    if err != nil {
        fmt.Fprintf(os.Stderr, "Component execution failed: %v\n", err)
        os.Exit(1)
    }
}
