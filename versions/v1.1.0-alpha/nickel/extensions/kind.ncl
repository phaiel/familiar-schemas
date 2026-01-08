# x-familiar-kind Extension Schema
# Defines the architectural role and its implications

{
  # Extension metadata
  extension = "x-familiar-kind",
  description = "Defines the architectural role of a schema",
  category = "identity",

  # JSON Schema definition
  schema = {
    type = "string",
    enum = [
      "node", "system", "action", "entity", "resource",
      "queue", "component", "ui", "tenant", "technique", "environment"
    ]
  },

  # Validation contract
  contract = std.contract.custom (fun label value =>
    if std.array.elem value [
      "node", "system", "action", "entity", "resource",
      "queue", "component", "ui", "tenant", "technique", "environment"
    ] then
      value
    else
      std.contract.blame_with label "Invalid x-familiar-kind: ${value}"
  ),

  # Code generation implications
  codegen = {
    rust = {
      derives = fun kind =>
        if kind == "entity" then
          ["Debug", "Clone", "Serialize", "Deserialize", "SeaORM"]
        else if kind == "component" then
          ["Debug", "Clone", "Serialize", "Deserialize", "PartialEq"]
        else
          ["Debug", "Clone", "Serialize", "Deserialize"],

      traits = fun kind =>
        if kind == "entity" then
          ["EntityTrait", "Related", "ActiveModelBehavior"]
        else if kind == "action" then
          ["Action", "Executable"]
        else
          []
    },

    typescript = {
      interfaces = fun kind =>
        if kind == "entity" then
          ["Entity", "Serializable"]
        else if kind == "component" then
          ["Component", "Composable"]
        else
          ["Serializable"]
    }
  },

  # Runtime implications
  runtime = {
    observability = fun kind => {
      metrics = kind != "component",  # Components usually don't need metrics
      tracing = kind == "action" || kind == "system",
      logging = true
    },

    lifecycle = fun kind =>
      if kind == "action" then
        "transient"  # Actions are short-lived
      else if kind == "system" then
        "singleton"  # Systems are long-lived
      else if kind == "entity" then
        "pooled"     # Entities may be pooled
      else
        "standard"
  },

  # Directory placement rules
  directory_rules = {
    allowed_in = fun kind =>
      if kind == "node" || kind == "system" || kind == "queue" then
        ["infrastructure"]
      else if kind == "entity" then
        ["entities"]
      else if kind == "ui" then
        ["ui"]
      else if kind == "component" then
        ["components"]
      else
        ["any"],

    primary_directory = fun kind =>
      if kind == "node" || kind == "system" || kind == "queue" then
        "infrastructure"
      else if kind == "entity" then
        "entities"
      else if kind == "ui" then
        "ui"
      else if kind == "component" then
        "components"
      else if kind == "action" then
        "infrastructure/actions"
      else if kind == "resource" then
        "infrastructure/resources"
      else
        "types"
  }
}
