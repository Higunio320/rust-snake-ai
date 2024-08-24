# Rust Snake AI

Rust Snake AI is an Evolutionary Artificial Neural Network (EANN) implemented in Rust designed to learn and play the Snake game. This project explores the performance benefits of Rust while implementing AI from scratch.

#### Important note
General idea, NN macroparameters and fitness function formula were heavily inspired by [Chrispresso's](https://www.youtube.com/@Chrispresso) video: [AI Learns to play Snake!](https://www.youtube.com/watch?v=vhiO4WsHA6c)

Snake game itself is a slightly modified version of [Snake game](https://github.com/ggez/ggez/blob/master/examples/04_snake.rs) from official [ggez repository](https://github.com/ggez/ggez) 

The rest of the code has been entirely written by myself.

## Overview

Rust Snake AI is a fast, multithreaded EANN designed to learn to play Snake game based on a simple Evolutionary Algorithm (EA) and Multilayer Perceptron (MLP).

EA specs:
* Selection type: **Roulette Wheel Selection (RWS)**
* Mutation type: **Normal Distribution Mutation**
* Population size: **500**
* Crossing probability: **0.9**
* Mutation probability: **0.3**
* Mutation range: **0.3**

MPL specs:
* Layers: **32, 20, 12, 4**
* Hidden layers activation function: **ReLU**
* Output layer activation function: **Softmax**

### Usage
Right now there are no parameters that can be modified from the command line.
After you cloned the repo just run
```bash
cargo run --release
```
and hope that the EA does it's work ;)

The EANN will train for 2000 generations, printing the generation number and the best fitness score for each. After training, a window will display a live visualisation of the Snake game starting from generation 1900. Use the right arrow key to skip to the next generation. It's important to note that the games presented in the window are played live, they aren't the games from the training phase. 

#### Example
Here you can see a gif visualising one of the best individuals I've been able to generate using my program.

![Snake game example gif](snake_game.gif)