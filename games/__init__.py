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
from .pacman import PacmanGame
from .flappy import FlappyGame
from .doodle import DoodleGame
from .game2048 import Game2048
from .minesweeper import MinesweeperGame
from .sudoku import SudokuGame
from .frogger import FroggerGame
from .memory import MemoryGame
from .solitaire import SolitaireGame
from .aimtrainer import AimTrainerGame
from .connect4 import ConnectFourGame
from .tanks import TankDuelGame
from .blackjack import BlackjackGame
from .tunnelracer import TunnelRacerGame
from .labyrinth import LabyrinthGame
from .reversi import ReversiGame
from .kniffel import KniffelGame
from .wordle import WordleGame
from .trexrunner import TRexRunnerGame
from .dame import DameGame
from .poker import PokerGame
from .chess import ChessGame
from .muehle import MuehleGame
from .simon import SimonGame
from .billiard import BilliardGame
from .slidepuzzle import SlidingPuzzleGame
from .mastermind import MastermindGame
from .bubbleshooter import BubbleShooterGame
from .hangman import HangmanGame
from .blockjump import BlockJumpGame
from .lamatowerdefense import LamaTowerDefenseGame

# Reihenfolge der Spiele im Menü
ALL_GAMES = [SnakeGame, PongGame, AirHockeyGame, TicTacToeGame, BreakoutGame,
             TetrisGame, InvadersGame, AsteroidsGame, PacmanGame, FlappyGame,
             DoodleGame, Game2048, MinesweeperGame, SudokuGame, FroggerGame,
             MemoryGame, SolitaireGame, AimTrainerGame, ConnectFourGame,
             TankDuelGame, BlackjackGame, TunnelRacerGame, LabyrinthGame,
             ReversiGame, KniffelGame, WordleGame,
             TRexRunnerGame, DameGame, PokerGame,
             ChessGame, MuehleGame, SimonGame, BilliardGame,
             SlidingPuzzleGame, MastermindGame, BubbleShooterGame, HangmanGame,
             BlockJumpGame, LamaTowerDefenseGame]
