# 🌐 Nyash WebAssembly Canvas Demos

A collection of interactive web applications demonstrating the **Everything is Box** philosophy in browser environments using WebAssembly.

## 🎯 Demo Applications

### 1. 🎨 Interactive Drawing App (`01_drawing_app.nyash`)
**Boxes Used:** `WebCanvasBox` + `CanvasEventBox` + `TimerBox`

- **Features:**
  - Click and drag drawing
  - Color palette selection
  - Brush size control
  - Real-time mouse tracking
- **Demonstrates:** Event handling, canvas drawing APIs, user interaction

### 2. ⏰ Clock & Timer (`02_clock_timer.nyash`)
**Boxes Used:** `TimerBox` + `WebCanvasBox`

- **Features:**
  - Digital clock display
  - Analog clock with hands
  - Stopwatch functionality
  - Real-time updates
- **Demonstrates:** Time management, animation loops, mathematical transformations

### 3. 🎆 Particle Fireworks (`03_particle_fireworks.nyash`)
**Boxes Used:** `CanvasLoopBox` + `RandomBox` + `WebCanvasBox`

- **Features:**
  - Physics-based particle system
  - Gravity and friction simulation
  - Random color generation
  - Automatic firework bursts
- **Demonstrates:** Game physics, particle systems, animation optimization

### 4. 🎲 Random Color Generator (`04_color_generator.nyash`)
**Boxes Used:** `RandomBox` + `WebCanvasBox`

- **Features:**
  - Color harmony algorithms (monochromatic, complementary, triadic)
  - HSL color space manipulation
  - Palette export functionality
  - Professional color theory implementation
- **Demonstrates:** Advanced algorithms, color science, data export

### 5. 🕹️ Mini Pong Game (`05_mini_pong.nyash`)
**Boxes Used:** `CanvasLoopBox` + `CanvasEventBox` + `WebCanvasBox` + `RandomBox`

- **Features:**
  - Two-player pong game
  - Ball physics with spin
  - AI opponent
  - Score tracking and win conditions
  - Collision detection
- **Demonstrates:** Game development, real-time gameplay, complex state management

## 🚀 Quick Start

### Option 1: View Demos in Browser
Open `canvas_demos.html` in your browser for an interactive experience with simulated Nyash functionality.

### Option 2: Run with WASM (Future)
```bash
# Build WASM package
cd projects/nyash-wasm
./build.sh

# Start local server
python3 -m http.server 8000

# Open demos
open http://localhost:8000/canvas_demos.html
```

## 📦 Box Architecture

### Core Canvas Boxes
- **`WebCanvasBox`**: HTML5 Canvas drawing operations
- **`CanvasEventBox`**: Mouse, touch, and keyboard input
- **`CanvasLoopBox`**: Animation frame management
- **`TimerBox`**: setTimeout/setInterval/requestAnimationFrame

### Supporting Boxes
- **`RandomBox`**: Random number generation, probability
- **`MathBox`**: Mathematical operations and constants

## 🎨 Technical Highlights

### Everything is Box Philosophy
```nyash
// Each component is a unified Box with consistent interface
local canvas, events, timer, random
canvas = new WebCanvasBox("my-canvas", 800, 600)
events = new CanvasEventBox("my-canvas")
timer = new TimerBox()
random = new RandomBox()

// All operations follow the same Box patterns
canvas.fillCircle(x, y, radius, color)
events.onMouseClick(callback)
timer.setTimeout(callback, delay)
color = random.choice(colorPalette)
```

### Advanced Features Demonstrated
- **Real-time Animation:** 60fps game loops with delta timing
- **Physics Simulation:** Gravity, friction, collision detection
- **Color Science:** HSL color space, harmony algorithms
- **Event Systems:** Mouse/keyboard input handling
- **State Management:** Game states, UI state, persistence

### Performance Optimizations
- Efficient particle system updates
- Canvas drawing batching
- Memory-conscious object pooling
- Delta time-based animations

## 🔧 Development Notes

### Adding New Demos
1. Create `XX_demo_name.nyash` in this directory
2. Use established Box patterns for consistency
3. Include comprehensive comments explaining Box usage
4. Add entry to `canvas_demos.html` for web testing

### Box Integration Guidelines
- Always use Box constructors: `new BoxName()`
- Follow naming conventions: `camelCase` for methods
- Include error handling for WASM/non-WASM environments
- Document Box interactions in code comments

### Browser Compatibility
- Tested on modern browsers with Canvas support
- WebAssembly required for full Nyash runtime
- Graceful fallback to JavaScript simulation

## 🌟 Future Enhancements

### Additional Demos Planned
6. **Audio Visualizer** - `AudioBox` + frequency analysis
7. **QR Code Generator** - `QRBox` + camera integration
8. **Real-time Chat** - `WebSocketBox` + multiplayer
9. **3D Graphics** - `WebGLBox` + 3D transformations
10. **Camera Effects** - `CameraBox` + image processing

### Advanced Box Features
- **`SpriteBox`**: Image loading and sprite animation
- **`ShapeBox`**: Complex geometric shapes
- **`TextDrawBox`**: Advanced typography
- **`ParticleBox`**: Professional particle effects
- **`AudioBox`**: Sound synthesis and playback

## 📖 Learning Resources

- **[Nyash Language Guide](../../docs/)** - Core language features
- **[Box Reference](../../docs/reference/built-in-boxes.md)** - Complete Box API
- **[WebAssembly Setup](../projects/nyash-wasm/README.md)** - WASM build instructions

---

**🐱 Everything is Box - even web applications!**

*These demos showcase how Nyash's unified Box architecture creates powerful, composable systems that work beautifully in web browsers through WebAssembly.*