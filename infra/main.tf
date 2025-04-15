variable "project_name" {
    type = string
}

variable "backend_bucket" {
    type = string
}

variable "backend_state_name" {
    type = string
}

variable "ssh_keys" {
    type = list(string)
}

variable "tailscale_auth_key" {
    type = string
}

variable "github_token" {
    type = string
}

terraform {
    required_version = "> 1.6.3"
    
    backend "s3" {
        endpoints = {
          s3 = "https://sfo3.digitaloceanspaces.com"
        }

        bucket = var.backend_bucket
        key    = var.backend_state_name

        skip_credentials_validation = true
        skip_requesting_account_id  = true
        skip_metadata_api_check     = true
        skip_region_validation      = true

        region                      = "us-east-1"
    }
}

data "digitalocean_project" "service_project" {
  name = var.project_name
}

resource "digitalocean_droplet" "service" {
  name    = "ovejas-webserver"

  image   = "debian-12-x64"
  region  = "sfo3"
  size    = "s-1vcpu-1gb"

  monitoring = true

  ssh_keys = var.ssh_keys

  user_data = <<-EOT
  #!/bin/bash
  apt update -y

  # Install tailscale 
  curl -fsSL https://tailscale.com/install.sh | sh
  tailscale up --auth-key "${var.tailscale_auth_key}"

  export HOME='/root'

  # Clone project
  apt install -y git
  git clone https://github.com/duskje/ovejas_project.git $HOME/ovejas_project

  # Install Rust
  apt install -y curl
  apt install -y build-essential
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source $HOME/.bashrc

  # Configure database
  export DATABASE_URL=$HOME/local.db
  export PORT='9734'
  export ADDRESS='0.0.0.0'

  apt install sqlite3 -y
  curl --proto '=https' --tlsv1.2 -LsSf https://github.com/diesel-rs/diesel/releases/latest/download/diesel_cli-installer.sh | sh

  cd $HOME/ovejas_project/server
  diesel migration run

  # Run project
  cargo run
  EOT
}

resource "digitalocean_project_resources" "project-services" {
  project = data.digitalocean_project.service_project.id

  resources = [
    digitalocean_droplet.service.urn
  ]
}

output "server_address" { 
  value = digitalocean_droplet.service.ipv4_address
}
