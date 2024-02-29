# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="bullseye"
FROM mcr.microsoft.com/devcontainers/rust:0-1-bullseye

# [Optional] Uncomment this section to install additional packages.
# RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
#     && apt-get -y install --no-install-recommends <your-package-list-here>
RUN apt-get update && apt-get install libclang-dev libhdf5-dev cmake -y
WORKDIR /tmp
RUN curl -L -O https://github.com/neovim/neovim/archive/refs/tags/v0.9.4.tar.gz
RUN tar -xvf v0.9.4.tar.gz

RUN cd neovim-0.9.4 && make CMAKE_BUILD_TYPE=Release && make install && mv ./build/bin/nvim /usr/bin/nvim
RUN rm -rf neovim-0.9.4
RUN rm v0.9.4.tar.gz
USER vscode
RUN git clone https://github.com/LazyVim/starter ~/.config/nvim
WORKDIR /workspace
