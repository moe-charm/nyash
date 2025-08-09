# 🐱 Nyash Programming Language
**Next-Generation Browser-Native Programming Experience**

*[🇯🇵 日本語版はこちら / Japanese Version](README.ja.md)*

[![Build Status](https://img.shields.io/badge/Build-Passing-brightgreen.svg)](#)
[![Everything is Box](https://img.shields.io/badge/Philosophy-Everything%20is%20Box-blue.svg)](#philosophy)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-Ready-orange.svg)](#webassembly)
[![Try Now](https://img.shields.io/badge/Try%20Now-Browser%20Playground-ff6b6b.svg)](projects/nyash-wasm/nyash_playground.html)
[![MIT License](https://img.shields.io/badge/License-MIT-green.svg)](#license)

---

## 🚀 **Try Nyash Right Now!**

**No installation, no setup - just open and code!**

👉 **[🎮 Launch Nyash Browser Playground](https://moe-charm.github.io/nyash/projects/nyash-wasm/nyash_playground.html)** 👈

Experience features like:
- 🎨 **Artist Collaboration Demo** - Multiple Box instances working together
- ⚡ **Async Computing** - Parallel processing made simple  
- 🎮 **Canvas Game Graphics** - Direct browser graphics programming
- 🔍 **Live Debug Visualization** - See your program's memory in real-time

---

## ✨ **Why Nyash Changes Everything**

### 🎯 **Memory Safety Revolution**
```nyash
// Traditional languages: manual memory management, crashes, security issues
// Nyash: Everything is Box - automatic, safe, elegant

static box Main {
    init { player, enemies, canvas }
    
    main() {
        me.player = new PlayerBox("Hero", 100)
        me.canvas = new WebCanvasBox("game", 800, 600)
        
        // Memory automatically managed - no crashes, no leaks!
        me.player.render(me.canvas)
        return "Game running safely!"
    }
}
```

### 🌐 **Browser-First Design**
- **Zero Installation**: Runs directly in web browsers via WebAssembly
- **Web APIs Built-in**: Canvas, DOM, storage - all native language features
- **Real-time Collaboration**: Share code instantly, run anywhere
- **Mobile Ready**: Works on phones, tablets, any modern device

### 🎨 **Creative Programming Made Easy**
```nyash
// Create art with code - naturally!
box Artist {
    init { name, color }
    
    paintMasterpiece(canvas) {
        canvas.fillCircle(100, 100, 50, me.color)
        canvas.fillText("Art by " + me.name, 10, 200, "24px Arial", me.color)
    }
}

// Multiple artists collaborate
picasso = new Artist("Picasso", "red")
monet = new Artist("Monet", "blue")
// Each Box maintains its own state and behavior!
```

### ⚡ **Async Simplicity**
```nyash
// Parallel processing without complexity
nowait future1 = heavyComputation(10000)
nowait future2 = renderGraphics()

// Do other work while they run...
setupUI()

// Get results when ready
result1 = await future1
result2 = await future2
```

---

## 🏗️ **Revolutionary Architecture**

### Everything is Box Philosophy
Every value in Nyash is a **Box** - a unified, memory-safe container:

| Traditional Languages | Nyash |
|----------------------|-------|
| `int x = 42;` | `x = new IntegerBox(42)` |
| `string name = "Hello";` | `name = new StringBox("Hello")` |
| Complex canvas setup | `canvas = new WebCanvasBox("game", 800, 600)` |
| Manual memory management | Automatic Box lifecycle management |

### Static Box Main Pattern
```nyash
// Clean, predictable program structure
static box Main {
    init { database, ui, gameState }  // Declare all fields upfront
    
    main() {
        // Initialize in logical order
        me.database = new DatabaseBox("save.db")
        me.ui = new UIManagerBox()
        me.gameState = new GameStateBox()
        
        // Your program logic here
        return runGameLoop()
    }
}
```

### Visual Debug Integration
```nyash
debug = new DebugBox()
debug.startTracking()

player = new PlayerBox("Hero")
debug.trackBox(player, "Main Character")

// Real-time memory visualization in browser!
print(debug.memoryReport())  // Live stats, no debugging hell
```

---

## 🎮 **Perfect for Creative Coding**

### Game Development
- **Built-in Canvas API**: Graphics without external libraries
- **Input Handling**: Mouse, keyboard, touch - all native
- **Audio Support**: SoundBox for music and effects
- **Physics Ready**: Mathematical operations optimized

### Educational Programming
- **Visual Feedback**: See your code's effects immediately
- **Memory Visualization**: Understand how programs work
- **No Setup Barriers**: Students code instantly in browser
- **Progressive Learning**: From simple scripts to complex applications

### Web Applications
- **Direct DOM Control**: WebDisplayBox manipulates HTML
- **No Framework Needed**: Language handles web interaction natively
- **Real-time Updates**: Changes reflect immediately
- **Cross-Platform**: Same code, everywhere

---

## 📖 **Language Highlights**

### Clean, Expressive Syntax
```nyash
// Object-oriented programming made natural
box Player {
    init { name, health, inventory }
    
    Player(playerName) {
        me.name = playerName
        me.health = 100
        me.inventory = new ArrayBox()
    }
    
    takeDamage(amount) {
        me.health = me.health - amount
        if me.health <= 0 {
            me.respawn()
        }
    }
    
    respawn() {
        me.health = 100
        print(me.name + " respawned!")
    }
}
```

### Powerful Operators
```nyash
// Natural language operators for clarity
isAlive = health > 0 and not poisoned
canCast = mana >= spellCost or hasItem("Magic Ring")
gameOver = playerDead or timeUp

// Mathematical operations built-in
distance = sqrt((x2 - x1)^2 + (y2 - y1)^2)
angle = atan2(deltaY, deltaX)
```

### Generic Programming
```nyash
// Type-safe generic containers
box Container<T> {
    init { value }
    
    Container(item) { me.value = item }
    getValue() { return me.value }
}

numbers = new Container<IntegerBox>(42)
texts = new Container<StringBox>("Hello")
```

---

## 🛠️ **Getting Started**

### Browser Development (Recommended)
```bash
# 1. Clone repository
git clone https://github.com/moe-charm/nyash.git
cd nyash

# 2. Build WebAssembly version
cd projects/nyash-wasm
./build.sh

# 3. Open playground in browser
# Open nyash_playground.html in any modern browser
```

### Native Development

#### Linux/WSL
```bash
# Build native version
cargo build --release

# Run programs locally
./target/release/nyash program.nyash

# Try examples
./target/release/nyash test_async_demo.nyash
./target/release/nyash app_dice_rpg.nyash
```

#### 🪟 Windows (Cross-compile)
```bash
# Install cross-compiler
cargo install cargo-xwin

# Build Windows executable
cargo xwin build --target x86_64-pc-windows-msvc --release

# Generated executable (916KB)
target/x86_64-pc-windows-msvc/release/nyash.exe
```

---

## 🤝 **Contributing**

Nyash is open source and welcomes contributions!

- **Issues**: Report bugs, request features
- **Pull Requests**: Code improvements, new examples
- **Documentation**: Help improve guides and examples
- **Community**: Share your Nyash creations!

## 📄 **License**

MIT License - Free for personal and commercial use.

---

## 🔗 **Links**

- **[🎮 Try Now - Browser Playground](https://moe-charm.github.io/nyash/projects/nyash-wasm/nyash_playground.html)**
- **[📚 Documentation](docs/)**
- **[🎯 Examples](examples/)**
- **[💬 Community Discussion](https://github.com/moe-charm/nyash/discussions)**

## 👨‍💻 **Creator**

**Moe Charm** - Programming Language Designer & Developer
- 🐙 GitHub: [@moe-charm](https://github.com/moe-charm)  
- 🐦 Twitter/X: [@CharmNexusCore](https://x.com/CharmNexusCore)
- ☕ Support Development: [coff.ee/moecharmde6](http://coff.ee/moecharmde6)

*Creating innovative programming languages with AI assistance and dedication 🤖*

---

## 🤖 **Support the Project**

Nyash is developed with cutting-edge AI collaboration! 

If you enjoy Nyash and want to support continued development:

**☕ [Support Development](http://coff.ee/moecharmde6)** - Help fuel innovation!

*Powered by Claude Code - Advanced AI development tools aren't free! 🤖*

Your support helps maintain the project, develop new features, and continue pushing the boundaries of programming language design. Every contribution makes a difference! 🙏

---

*Built with ❤️, 🤖 Claude Code, and the Everything is Box philosophy*

**Nyash - Where every value is a Box, and every Box tells a story.**