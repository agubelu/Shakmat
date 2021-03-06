# Plays several matches between two versions of Shakmat using different openings,
# and reports on the final results, both in term of scores and average move speed.
import requests as rq
from datetime import datetime
from json import dumps
from os.path import isfile
from os import remove
import sys
import chess
import chess.pgn
import colorama
from colorama import Fore, Back, Style

colorama.init()

OLD_VER = {"port": int(sys.argv[2]), "name": sys.argv[1]}
NEW_VER = {"port":  int(sys.argv[4]), "name": sys.argv[3]}

TOTAL_TIME = int(sys.argv[5]) * 1000 # ms
INCREMENT = int(float(sys.argv[6]) * 1000) # ms

FILE_PGN = "out/games.pgn"

class ShakmatVer:
    def __init__(self, port, name):
        self.name = name
        self.port = port
        self.current_game = None
        self.move_speeds = {}
        self.scores = {k: {s: 0 for s in ("win", "lose", "draw")} for k in ("black", "white")}
        self.timer = TOTAL_TIME

    def create_game(self):
        resp = rq.post(f"http://127.0.0.1:{self.port}/games").json()
        self.current_game = resp["key"]

    def delete_game(self):
        resp = rq.delete(f"http://127.0.0.1:{self.port}/games/{self.current_game}")
        assert resp.status_code == 204
        self.current_game = None

    def make_move(self, move):
        url = f"http://127.0.0.1:{self.port}/games/{self.current_game}/move"
        resp = rq.post(url, json={"move": move})
        if resp.status_code != 200: 
            print(move)
            print(resp.text)
        assert resp.status_code == 200
        return resp.json()["turn_info"]

    def get_best_move(self):
        url = f"http://127.0.0.1:{self.port}/games/{self.current_game}/move_suggestion?total_ms={int(self.timer)}"
        return rq.get(url).json()["move"]

    def update_score(self, color, result):
        assert color in ("black", "white")
        assert result in ("win", "lose", "draw")
        self.scores[color][result] += 1

    def update_moving_time(self, ply, time):
        self.move_speeds[ply] = self.move_speeds.get(ply, []) + [time]


class Match:
    def __init__(self, white, black, opening):
        self.white = white
        self.black = black
        self.opening_line = opening
        self.ply = 0

        self.game = chess.pgn.Game()
        self.game.headers["White"] = white.name
        self.game.headers["Black"] = black.name
        self.last_node = None

        self.white.create_game()
        self.black.create_game()

    def add_move(self, color_moving, move):
        last_node = self.last_node if self.last_node is not None else self.game
        next_node = last_node.add_variation(chess.Move.from_uci(move_to_uci(move, color_moving)))
        self.last_node = next_node

    def play(self):
        self.white.timer = TOTAL_TIME
        self.black.timer = TOTAL_TIME
        result = None
        while True:
            (moving_player, other_player) = (self.white, self.black) if self.ply % 2 == 0 \
                                            else (self.black, self.white)
            moving_color = "white" if self.ply % 2 == 0 else "black"
            
            move = None
            if self.ply < len(self.opening_line):
                # Inside the opening line, grab the best move from there
                move = self.opening_line[self.ply]
            else:
                # Not inside the opening line, ask the engine for the best move
                # and log the time it takes to reply
                t1 = datetime.now()
                move = moving_player.get_best_move()
                t2 = datetime.now()
                elapsed_ms = (t2-t1).total_seconds() * 1000
                moving_player.timer -= elapsed_ms
                moving_player.update_moving_time(self.ply, elapsed_ms / 1000)

                if moving_player.timer > 0:
                    moving_player.timer += INCREMENT
            
            if move:
                # Make the move on both sides
                moving_player.make_move(move)
                turn_info = other_player.make_move(move)
                self.add_move(moving_color, move)
                self.ply += 1
            else:
                turn_info = {"moves": []}

            # Check if the player lost by time
            if moving_player.timer <= 0:
                if moving_player is self.white:
                    print("White ran out of time")
                    self.white.update_score("white", "lose")
                    self.black.update_score("black", "win")
                    result = "B"
                else:
                    print("Black ran out of time")
                    self.white.update_score("white", "win")
                    self.black.update_score("black", "lose")
                    result = "W"

                break
            
            if not turn_info["moves"] or not move:
                # No moves available, check whether this is checkmate or a draw
                if turn_info["in_check"]:
                    # Checkmate
                    if moving_player is self.white:
                        # White checkmated black
                        self.white.update_score("white", "win")
                        self.black.update_score("black", "lose")
                        self.game.headers["Result"] = "1-0"
                        result = "W"
                    else:
                        # Black checkmated white
                        self.white.update_score("white", "lose")
                        self.black.update_score("black", "win")
                        self.game.headers["Result"] = "0-1"
                        result = "B"
                else:
                    # Draw
                    self.white.update_score("white", "draw")
                    self.black.update_score("black", "draw")
                    self.game.headers["Result"] = "1/2-1/2"
                    result = "D"
                
                break

        self.white.delete_game()
        self.black.delete_game()

        with open(FILE_PGN, "a") as f:
            f.write(str(self.game) + "\n\n")

        return result

def move_to_uci(move, color):
    move = move.lower()
    if "=" in move:
        return move.replace("=", "")
    elif move == "o-o":
        return {
            "white": "e1g1",
            "black": "e8g8"
        }[color]
    elif move == "o-o-o":
        return {
            "white": "e1c1",
            "black": "e8c8"
        }[color]
    else:
        return move

old_engine = ShakmatVer(OLD_VER["port"], OLD_VER["name"])
new_engine = ShakmatVer(NEW_VER["port"], NEW_VER["name"])

openings = [[]] + [line.strip().split(" ") for line in open("openings.txt", "r").readlines()]

if isfile(FILE_PGN):
    remove(FILE_PGN)

for i, opening_line in enumerate(openings, start=1):
    print(f"Opening {i}, game 1... ", end="", flush=True)
    res = Match(old_engine, new_engine, opening_line).play()
    d = {"W": Fore.RED + OLD_VER["name"], "B": Fore.GREEN + NEW_VER["name"], "D": Fore.YELLOW + "Draw"}
    print(d[res]+ Style.RESET_ALL, flush=True)

    print(f"Opening {i}, game 2... ", end="", flush=True)
    res = Match(new_engine, old_engine, opening_line).play()
    d = {"B": Fore.RED + OLD_VER["name"], "W": Fore.GREEN + NEW_VER["name"], "D": Fore.YELLOW + "Draw"}
    print(d[res] + Style.RESET_ALL, flush=True)

with open("out/scores.json", "w") as f:
    f.write(dumps({OLD_VER["name"]: old_engine.scores, NEW_VER["name"]: new_engine.scores}))

with open("out/times.json", "w") as f:
    f.write(dumps({OLD_VER["name"]: old_engine.move_speeds, NEW_VER["name"]: new_engine.move_speeds}))