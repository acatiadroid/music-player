import tkinter as tk
from tkinter import messagebox
from tkinter.font import Font
from tkinter.colorchooser import askcolor

from .config import *

# These remain cosntant as we do not want to make the colour
# of the settings window customizable
FORE_COLOUR = "#2b2b2b"
BACK_COLOUR = "#111111"


def pick_colour(*, bg=False, fg=False, accent=False, songlist=False):
    clr = askcolor(color=None)
    _, hex_colour = clr

    if not hex_colour:
        return

    if bg:
        write("back_colour", hex_colour)
    if fg:
        write("fore_colour", hex_colour)
    if accent:
        write("accent_colour", hex_colour)
    if songlist:
        write("songlist_colour", hex_colour)

    messagebox.showinfo("Settings Updated", "Settings have been saved!\n\nYou will need to restart the music player for the settings to be applied.")


def settings_window():
    window = tk.Toplevel()
    window.wm_title("Settings")
    window.configure(bg=view("back_colour"))
    window.geometry("400x500")
    window.resizable(False, False)
    window.wm_iconbitmap("music_player/Assets/settings.ico")

    # Labels

    font = Font(size=11, family="Cascadia Mono")

    tk.Label(
        window,
        font=Font(size=13, family="Cascadia Mono", weight="bold"),
        text="Settings",
        fg="white",
        bg=BACK_COLOUR
    ).grid(sticky="n")

    # Background
    
    tk.Label(
        window,
        text="Background Colour:",
        font=font,
        bg=BACK_COLOUR,
        fg="white"
    ).grid(row=3, column=0, sticky="e")

    current_bg = tk.Entry(
        window,
        bg=BACK_COLOUR,
        fg="white",
        width=10,
        font=font,
    )
    current_bg.grid(row=3, column=1)
    current_bg.insert(0, view("back_colour"))

    change_bg = tk.Button(
        window,
        text="Change",
        command=lambda: pick_colour(bg=True),
        borderwidth=0,
        fg="white",
        bg=FORE_COLOUR,
        font=Font(size=10, family="Cascadia Mono")
    )
    change_bg.grid(row=3, column=2)

    # Foreground
    
    tk.Label(
        window,
        text="Foreground Colour:",
        font=font,
        bg=BACK_COLOUR,
        fg="white"
    ).grid(row=4, column=0, sticky="e")

    current_fg = tk.Entry(
        window,
        bg=BACK_COLOUR,
        fg="white",
        width=10,
        font=font,
    )
    current_fg.grid(row=4, column=1)
    current_fg.insert(0, view("fore_colour"))

    change_fg = tk.Button(
        window,
        text="Change",
        command=lambda: pick_colour(fg=True),
        borderwidth=0,
        fg="white",
        bg=FORE_COLOUR,
        font=Font(size=10, family="Cascadia Mono")
    )
    change_fg.grid(row=4, column=2)

    # Accent

    tk.Label(
        window,
        text="Accent Colour:",
        font=font,
        bg=BACK_COLOUR,
        fg="white"
    ).grid(row=5, column=0, sticky="e")

    current_fg = tk.Entry(
        window,
        bg=BACK_COLOUR,
        fg="white",
        width=10,
        font=font,
    )
    current_fg.grid(row=5, column=1)
    current_fg.insert(0, view("accent_colour"))

    change_fg = tk.Button(
        window,
        text="Change",
        command=lambda: pick_colour(accent=True),
        borderwidth=0,
        fg="white",
        bg=FORE_COLOUR,
        font=Font(size=10, family="Cascadia Mono")
    )
    change_fg.grid(row=5, column=2)

    # Songlist colour

    tk.Label(
        window,
        text="Song list Colour:",
        font=font,
        bg=BACK_COLOUR,
        fg="white"
    ).grid(row=6, column=0, sticky="e")

    current_songlist = tk.Entry(
        window,
        bg=BACK_COLOUR,
        fg="white",
        width=10,
        font=font,
    )
    current_songlist.grid(row=6, column=1)
    current_songlist.insert(0, view("songlist_colour"))

    change_songlist = tk.Button(
        window,
        text="Change",
        command=lambda: pick_colour(accent=True),
        borderwidth=0,
        fg="white",
        bg=FORE_COLOUR,
        font=Font(size=10, family="Cascadia Mono")
    )
    change_songlist.grid(row=6, column=2)