import os

import tkinter as tk

import player.utils.data as data
import player.utils.constants as constants
import player.widgets as widgets

class Playlists:
    def __init__(self, win_properties):
          self.pos = win_properties
          self.images = constants.Image()
          self.fonts = constants.Font()
     
    def show_playlists(self):
        """Retrieves list of playlists"""
        window = tk.Toplevel()
        window.wm_title("Your playlists")
        window.configure(bg=data.view("back_colour", "c"))
        window.geometry(f"500x500+{self.pos[0]}+{self.pos[1]}")
        window.resizable(False, False)
        data.get_playlists()
        
        window.sbf = widgets.ScrollbarFrame(window)
        window.grid_rowconfigure(0, weight=1)
        window.grid_columnconfigure(0, weight=1)
        window.sbf.grid()

        window.scroll_frame = window.sbf.scrolled_frame

    def create_playlist(self):
        """Creates a new, empty playlist."""
        window = tk.Toplevel()
        window.wm_title("Create new playlist")
        window.configure(bg=data.view("back_colour", "c"))
        window.resizable(False, False)
        
        if len([f for f in os.listdir("./data/audio/") if f.endswith(".mp3")]) == 0:
            tk.Label(
                window,
                text="You don't have any playlists",
                bg=data.view("songlist_colour", "c"),
                fg="white",
                font=self.fonts.MAIN
            ).grid(row=0, column=0)
            return
        
        for file in os.listdir("./data/audio/"):
                if not file.endswith(".mp3"):
                    continue

                i += 1
                    
                if len(file[:-4]) > 40: # Reduce length of song name to prevent duration from being pushed off the screen.
                    to_chop = len(file[:-4]) - 40
                    file = file[:-to_chop] + "..."
                else:
                    file = file[:-4] # remove ".mp3"
                btn = widgets.HoverButton(
                    self.scroll_frame,
                    text="[Load]",
                    borderwidth=0,
                    bg=data.view("songlist_colour", "c"),
                    font=self.fonts.MAIN,
                    fg=data.view("accent_colour", "c"),
                    command=self.open_playlist,
                    activebackground=data.view("songlist_colour", "c"),
                    activeforeground="white"
                )
                btn.grid(row=i, column=0)
                tk.Label(
                    self.scroll_frame,
                    text=file,
                    bg=data.view("songlist_colour", "c"),
                    font=self.fonts.MAIN,
                    fg="white",
                ).grid(row=i, column=1, sticky="w")

                tk.Label(
                    self.scroll_frame,
                    text="        " + duration,
                    bg=data.view("songlist_colour", "c"),
                    font=self.fonts.MAIN,
                    fg="white"
                ).grid(row=i, column=2, sticky="e")
                
        


    def open_playlist(self, plistid):
        """Opens a playlist"""