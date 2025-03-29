<img src="/.github/assets/demo.gif" alt="Demo">

## 📄 About
A fast, concurrent CSV downloader for World of Warcraft DB2 data from [wago.tools](https://wago.tools).

## 🚀 Features
- 🚀 Concurrent downloads (up to 4 files simultaneously by default)
- 📊 Progress tracking 
- 🔄 Automatic retry on failures
- 📦 Skip existing files
- 🌍 Support for multiple locales
- 🎮 Interactive build selection
- ⚡ Rate limiting to prevent server overload

Pour compiler.
Dans le terminal:
cargo clean

Ajouter la cible 32 bits MSVC
Dans votre terminal, exécutez : rustup target add i686-pc-windows-msvc

Compiler pour la cible 32 bits
Ensuite, compilez en spécifiant la cible : cargo build --release --target=i686-pc-windows-msvc

Cela créera l'exécutable dans le dossier : cargo build --release --target=i686-pc-windows-msvc

l'exe est mis dans
target\i686-pc-windows-msvc\release\

