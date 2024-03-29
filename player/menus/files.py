import os
import logging

import tkinter as tk
from tkinter import messagebox
import tkinter.filedialog as filedialog

import player.utils.data as data
import player.utils.constants as constants

_log = logging.getLogger("app.files")

class Files:
    def __init__(self, win_properties):
        self.pos = win_properties
        self.fonts = constants.Font()
        self.images = constants.Image()

    def open_filedialog(self, split=False):
        file = filedialog.askopenfile(
            initialdir="./data/audio/",
            title="Select an MP3 file",
            filetypes=(
                ("MP3 files", "*.mp3"),
                ("all files", "*.*"),
            )
        )
        if not file:
            return None

        if split:
            file = file.name.split("/")
            file = file[len(file)-1]

        return file

    def rename_window(self):
        file = self.open_filedialog(split=True)
        
        if not file:
            return 
            
        root = tk.Toplevel()
        root.configure(bg=data.view("back_colour", "c"))
        root.geometry(f"450x300+{self.pos[0]}+{self.pos[1]}")
        root.wm_title("Rename a file")
        root.resizable(True, False)
        
        try:
            root.iconbitmap(self.images.PENCIL)
        except tk.TclError:
            pass
        
        tk.Label(
            root,
            text="Rename a file",
            font=self.fonts.LARGE,
            fg="white",
            bg=data.view("back_colour", "c")
        ).pack()

        tk.Label(
            root,
            text="Selected song",
            font=self.fonts.MAIN,
            fg="white",
            bg=data.view("back_colour", "c")
        )

        selected_song = tk.Label(
            root,
            text=file,
            fg="#2ca351",
            bg=data.view("back_colour", "c"),
            font=self.fonts.MEDIUM
        ).pack()

        tk.Label(
            root, 
            text="Enter new name: (don't include .mp3)",
            fg="white", 
            bg=data.view("back_colour", "c"), 
            font=self.fonts.MAIN
        ).pack(pady=10)

        new_name = tk.Entry(
            root,
            fg="white",
            bg=data.view("back_colour", "c"),
            font=self.fonts.MAIN
        )
        new_name.pack()

        def rename():
            os.rename(f"./data/audio/{file}", "./data/audio/" + new_name.get() + ".mp3")
            root.destroy()
            messagebox.showinfo(
                title="Success!",
                message="File renamed successfully."
            )

        btn = tk.Button(
            root,
            fg="white",
            bg=data.view("fore_colour", "c"),
            text="Done",
            font=self.fonts.MAIN,
            command=rename
        )
        btn.pack(pady=20)

    def delete_file(self):
        file = self.open_filedialog(split=True)

        if not file:
            return

        os.remove("./data/audio/" + file)

        messagebox.showinfo(
            title="File deleted",
            message=f"Successfully deleted \"{file}\""
        )