job "clrinfjs-inspector" {
  datacenters = ["dc1"]
  type        = "service"

  group "dev-tools" {
    count = 1

    network {
      port "http" {
        to = 4200
      }
    }

    service {
      name = "clrinfjs-event-inspector"
      port = "http"

      check {
        type     = "http"
        path     = "/health"
        interval = "15s"
        timeout  = "2s"
      }
    }

    task "server" {
      driver = "docker"

      config {
        image = "clrinf/clrinfjs-inspector:latest"
        ports = ["http"]
      }

      env {
        PORT     = "4200"
        HOST     = "0.0.0.0"
        NODE_ENV = "development"
      }

      # Populate this Nomad variable before deploying; never bake credentials into the image.
      template {
        data = <<EOH
{{ with nomadVar "nomad/jobs/clrinfjs-inspector" }}
INSPECTOR_TOKEN={{ .token | toJSON }}
{{ end }}
EOH
        destination = "secrets/inspector.env"
        env         = true
      }

      resources {
        cpu    = 256
        memory = 256
      }
    }
  }
}
