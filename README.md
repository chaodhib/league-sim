# League Sim

A simulator for League of Legends champion abilities and item combinations, focusing on optimizing damage output. Currently supports Kha'Zix with plans to expand to other champions.

## Features

- **Item Optimizer**: Find the best item combinations for maximum damage output
- **Combo Optimizer**: Discover optimal ability sequences for your chosen items
- **Single Simulation**: Test specific item and ability combinations
- **Detailed Configuration**:
  - Champion settings (level, health, champion specific configs)
  - Target settings (armor, magic resistance, health)
  - Rune settings
  - Game state settings (crit handling, game time)
  - Item-specific configurations

## Project Structure

- `league-sim-frontend/`: Vue.js frontend application.
- `league-sim-backend/`: Simulation code written in Rust which gets compiled into WebAssembly.
- `game-data-parser/`: Tool for parsing League of Legends game data. The output is then used by the backend simulation code.
- `shared-structs/`: Shared data structures between the data parser and the simulation backend.

## Getting Started

### Prerequisites

- Node.js and npm for the frontend
- Rust and Cargo for the backend
- League of Legends game files for data parsing (optional, only needed if you want to update game data)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/chaodhib/league-sim.git
cd league-sim
```

2. Compile the WebAssembly package:
```bash
cd league-sim-backend
wasm-pack build
```

3. Set up the frontend:
```bash
cd league-sim-frontend
npm install
```

### Running the Application

1. Start the frontend development server:
```bash
cd league-sim-frontend
npm run dev
```

## Contributing

Contributions are welcome! Here's how you can help:

1. **Report Bugs**: Open an issue describing the bug and how to reproduce it
2. **Suggest Features**: Open an issue describing your feature idea
3. **Submit Pull Requests**: Fork the repo and create a pull request with your changes
4. **Add Champion Support**: Help expand the simulator to support more champions
5. **Improve Documentation**: Help make our docs better and more comprehensive

### Development Guidelines

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

- Frontend: Follow Vue.js style guide
- Backend: Follow Rust style guidelines
- Use meaningful variable names and add comments for complex logic
- Write tests for new features

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

A big thanks to:
- [The Meraki project](https://github.com/meraki-analytics/lolstaticdata) from which we pull game data.
- [Cdragon project](https://github.com/CommunityDragon/Data) from which we pull game data.
- The League of Legends Wiki contributors
- The community and project contributors.

## Contact

Project Link: [https://github.com/chaodhib/league-sim](https://github.com/chaodhib/league-sim)

Feel free to reach out if you have questions or want to contribute!
