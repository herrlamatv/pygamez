# -*- coding: utf-8 -*-
"""Paket mit allen Spielen der Sammlung."""

from .snake import SnakeGame
from .pong import PongGame
from .tictactoe import TicTacToeGame
from .breakout import BreakoutGame
from .tetris import TetrisGame
from .invaders import InvadersGame
from .game2048 import Game2048

# Reihenfolge der Spiele im Menue
ALL_GAMES = [SnakeGame, PongGame, TicTacToeGame, BreakoutGame,
             TetrisGame, InvadersGame, Game2048]
