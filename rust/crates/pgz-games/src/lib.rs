//! pgz-games
//! =========
//! Die Spiele - je Datei eine Python-Vorlage aus `games/`.
//!
//! `all_games()` haelt die Reihenfolge von `games/__init__.py` (`ALL_GAMES`).
//! Diese Datei wird geschrieben, nicht von Hand gepflegt.

// approx_constant: Zahlen wie 6.28 oder 0.7071 stehen genau so in der
// Python-Vorlage - eine genauere Konstante gaebe andere Pixel.
#![allow(clippy::new_ret_no_self, clippy::too_many_arguments, clippy::approx_constant)]

use pgz_engine::GameInfo;

pub mod aimtrainer;
pub mod airhockey;
pub mod asteroids;
pub mod battleship;
pub mod battleship_core;
pub mod billiard;
pub mod blackjack;
pub mod blockjump;
pub mod bowling;
pub mod breakout;
pub mod bubbleshooter;
pub mod cards;
pub mod casino;
pub mod casino_draw;
pub mod casino_logic;
pub mod casino_roulette;
pub mod casino_slots;
pub mod chess;
pub mod chess_draw;
pub mod chess_engine;
pub mod competitive;
pub mod connect4;
pub mod crossyroad;
pub mod crossyroad_draw;
pub mod crossyroad_models;
pub mod crossyroad_world;
pub mod dame;
pub mod data;
pub mod doodle;
pub mod flappy;
pub mod frogger;
pub mod game2048;
pub mod geodash;
pub mod geodash_core;
pub mod geodash_draw;
pub mod geodash_edit;
pub mod geodash_extra;
pub mod geodash_music;
pub mod hangman;
pub mod invaders;
pub mod kniffel;
pub mod labyrinth;
pub mod lamacrash;
pub mod lamacrash_draw;
pub mod lamacrash_logic;
pub mod lamacrash_music;
pub mod lamacrash_world;
pub mod lamacrossing;
pub mod lamacrossing_draw;
pub mod lamacrossing_logic;
pub mod lamamines;
pub mod lamamines_draw;
pub mod lamamines_logic;
pub mod lamaplinko;
pub mod lamaplinko_draw;
pub mod lamaplinko_logic;
pub mod lamatowerdefense;
pub mod mastermind;
pub mod matchtext;
pub mod maze_gen;
pub mod memory;
pub mod minesweeper;
pub mod minigolf;
pub mod minigolf_draw;
pub mod minigolf_edit;
pub mod minigolf_gen;
pub mod muehle;
pub mod ngb;
pub mod pacman;
pub mod pinball;
pub mod poker;
pub mod pong;
pub mod prestige;
pub mod reversi;
pub mod simon;
pub mod slidepuzzle;
pub mod snake;
pub mod solitaire;
pub mod sudoku;
pub mod sudoku_gen;
pub mod tanks;
pub mod tetris;
pub mod tetris_ai;
pub mod tetris_core;
pub mod tictactoe;
pub mod trexrunner;
pub mod tunnelracer;
pub mod ugc;
pub mod wordle;
pub mod wordle_words;

/// Alle uebertragenen Spiele in der Reihenfolge von `ALL_GAMES`.
pub fn all_games() -> Vec<GameInfo> {
    vec![
        snake::INFO,
        pong::INFO,
        airhockey::INFO,
        tictactoe::INFO,
        breakout::INFO,
        tetris::INFO,
        invaders::INFO,
        asteroids::INFO,
        pacman::INFO,
        flappy::INFO,
        doodle::INFO,
        game2048::INFO,
        minesweeper::INFO,
        sudoku::INFO,
        frogger::INFO,
        memory::INFO,
        solitaire::INFO,
        aimtrainer::INFO,
        connect4::INFO,
        tanks::INFO,
        blackjack::INFO,
        tunnelracer::INFO,
        labyrinth::INFO,
        reversi::INFO,
        kniffel::INFO,
        wordle::INFO,
        trexrunner::INFO,
        dame::INFO,
        poker::INFO,
        chess::INFO,
        muehle::INFO,
        simon::INFO,
        billiard::INFO,
        slidepuzzle::INFO,
        mastermind::INFO,
        bubbleshooter::INFO,
        hangman::INFO,
        blockjump::INFO,
        lamatowerdefense::INFO,
        minigolf::INFO,
        pinball::INFO,
        bowling::INFO,
        crossyroad::INFO,
        geodash::INFO,
        battleship::INFO,
        casino::INFO,
        lamacrossing::INFO,
        lamamines::INFO,
        lamaplinko::INFO,
        lamacrash::INFO,
    ]
}
