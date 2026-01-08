# Infrastructure Queues Directory - Inherits from infrastructure
(import "../_directory.ncl") & {
  # Queues manage async message flow

  required_extensions = (import "../_directory.ncl").required_extensions @ [
    "x-familiar-retention"  # Queues need retention policies
  ],

  hydration = (import "../_directory.ncl").hydration & {
    "_operations" = (import "../_directory.ncl").hydration."_operations" & {
      monitoring_category = "queue",
      message_tracking = true
    }
  }
}
