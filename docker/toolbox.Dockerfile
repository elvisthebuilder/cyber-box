# cyber-box toolbox image: Tor + the v1 pentest toolset, driven by the Rust
# TUI via `docker exec`. Headless — no X11, so tshark (not wireshark GUI).

FROM golang:1.22-bookworm AS gobuilder
ENV CGO_ENABLED=0
# Let go fetch a newer toolchain automatically if a dependency requires one
# newer than this base image ships (e.g. httpx currently requires go >= 1.26).
ENV GOTOOLCHAIN=auto
RUN go install github.com/projectdiscovery/httpx/cmd/httpx@latest \
    && go install github.com/ffuf/ffuf/v2@latest

FROM kalilinux/kali-rolling

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    nmap \
    masscan \
    whois \
    dnsutils \
    nikto \
    sqlmap \
    gobuster \
    dirb \
    hydra \
    john \
    hashcat \
    metasploit-framework \
    tor \
    torsocks \
    proxychains4 \
    tshark \
    supervisor \
    wordlists \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Kali ships rockyou.txt.gz compressed; unpack it if present so hydra/john/
# hashcat default flags in registry/tools.toml resolve out of the box.
RUN if [ -f /usr/share/wordlists/rockyou.txt.gz ]; then \
      gunzip -k /usr/share/wordlists/rockyou.txt.gz; \
    fi

COPY --from=gobuilder /go/bin/httpx /usr/local/bin/httpx
COPY --from=gobuilder /go/bin/ffuf /usr/local/bin/ffuf

COPY supervisord.conf /etc/cyberbox/supervisord.conf
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Tor stays OFF until the TUI enables it via `supervisorctl start tor`.
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
