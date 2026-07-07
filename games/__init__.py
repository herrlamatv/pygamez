# -*- coding: utf-8 -*-
"""Paket mit allen Spielen der Sammlung."""

from .snake import SnakeGame
from .pong import PongGame
from .airhockey import AirHockeyGame
from .tictactoe import TicTacToeGame
from .breakout import BreakoutGame
from .tetris import TetrisGame
from .invaders import InvadersGame
from .asteroids import AsteroidsGame
from .game2048 import Game2048
from .minesweeper import MinesweeperGame

# Reihenfolge der Spiele im Menü
ALL_GAMES = [SnakeGame, PongGame, AirHockeyGame, TicTacToeGame, BreakoutGame,
             TetrisGame, InvadersGame, AsteroidsGame, Game2048, MinesweeperGame]
