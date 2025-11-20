FROM mcr.microsoft.com/devcontainers/base:ubuntu

# Fix issue with starship-rs on bash not wrapping long commands to newline
RUN export DEBIAN_FRONTEND=noninteractive && apt-get update && apt-get install -y --no-install-recommends locales
RUN locale-gen "en_US.UTF-8"
ENV LANG="en_US.UTF-8"

RUN curl -sSL https://starship.rs/install.sh | sh -s -- --yes

USER vscode
