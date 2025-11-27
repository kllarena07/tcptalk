# TCPTalk

## About

A neat little party trick that you can show to your developer friends is that you can chat with each other in the terminal using [netcat](https://en.wikipedia.org/wiki/Netcat).

All you have to do is setup the server (`nc -l 0.0.0.0 3000`) and connect to it via the client (`nc 0.0.0.0 3000`).

This is pretty cool but the fun soon fades when you try to chat with more people, as only one-to-one connections are supported. So, chatting with more than two people at a time is not possible.

![EI292CUR](https://github.com/user-attachments/assets/8340d93e-0c1b-4cff-9adf-742f26578f0f)
> A demonstration of communicating between two parties using netcat.

This repository is a small little experiment that decides to expand on this netcat party trick by providing a server that supports multiple connections at once and a fully fledged TUI client program that connects to said server. In essence, a TCP chatroom for you and your friends.

https://github.com/user-attachments/assets/34b2399a-606d-4d51-9d3d-206ffa279c5a

Note: this repository is not built on top of `netcat`

## Setup
**A Couple Things to Note**
- Rust must be must be installed on your machine. You can can find out how to do that [here](https://rust-lang.org/tools/install/).
- This installation setup process was written that your platform is MacOS/Linux. If you are Windows, please consider using WSL or the installation may be different.

### Running the Server
1. Clone this repository: `git clone https://github.com/kllarena07/tcptalk`
2. Run the server
```
cd server/
cargo run --release
```
### Running the Client
1. Clone this repository: `git clone https://github.com/kllarena07/tcptalk`
2. Run the setup script
```
source ./setup_client.sh
```
3. Use the `tcptalk` command

The `tcptalk` command takes an argument of [username] [ip_address] [-p port]. This command will not be exported globally until you add it to your PATH.

Examples:
- `tcptalk alice` connects to 0.0.0.0:2133
- `tcptalk alice 127.0.0.1` connects to 127.0.0.1:2133
- `tcptalk alice 192.168.1.100 -p 9090` connects to 192.168.1.100:9090

## 👾 Bugs or vulnerabilities

If you find any bugs or vulnerabilities, please contact me on my Twitter using the link below.

_Made with ❤️ by [krayondev](https://x.com/krayondev)_
