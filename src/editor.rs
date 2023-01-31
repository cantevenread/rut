use std::fs::{File, OpenOptions};
use std::io::{stdout, Write};

use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
#[allow(unused_imports)]
use crossterm::style::{Print, PrintStyledContent};
use crossterm::terminal;
use crossterm::Result;
use crossterm::{execute, queue};
use ropey::Rope;

// Represents the state of the editor
// There should only be one instance of this struct at any given point
#[allow(dead_code)]
pub struct Editor {
    file: File,
    buffer: Rope,
    window_length: usize,
    window_height: usize,
}

impl Editor {
    // Create a new editor instance
    pub fn new(filename: &str) -> Self {
        // Open the file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename)
            .expect("[INTERNAL ERROR] Failed to open file");

        // Read the file into a Rope
        let buffer = Rope::from_reader(&file).unwrap();

        // Get the terminal size
        let window_size = terminal::size().expect("[INTERNAL ERROR] Failed to get terminal size");

        Self {
            file,
            buffer,
            window_length: window_size.0 as usize,
            window_height: window_size.1 as usize,
        }
    }

    // Opens the editor in the terminal and runs the event loop
    pub fn run(&mut self) -> Result<()> {
        // Enable raw mode for the terminal
        terminal::enable_raw_mode()?;

        // Clear the screen and draw the buffer
        self.update(true)?;

        // Start the event loop
        self.start_event_loop()
    }

    // Enters the event loop for the editor
    fn start_event_loop(&mut self) -> Result<()> {
        loop {
            // Wait for the next event
            // * This is a blocking call
            let event = event::read()?;

            // Dispatch the event to the appropriate handler
            self.handle_event(event)?;
        }
    }

    // Handles a generic Event by dispatching it to the appropriate handler function
    // ? Should this be done with nested functions?
    fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key_event) => self.handle_key_event(key_event)?,
            _ => (),
        }

        Ok(())
    }

    // Handles a KeyEvent using its code and modifiers
    fn handle_key_event(&mut self, event: KeyEvent) -> Result<()> {
        match (event.code, event.modifiers) {
            // Exit the program on Ctrl+C
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.exit().unwrap();
            }
            _ => (),
        }

        Ok(())
    }

    // [Direct/Lazy] Clears the screen
    fn clear_screen(&self, keep_cursor_pos: bool, direct_execute: bool) -> Result<()> {
        queue!(stdout(), terminal::Clear(terminal::ClearType::All))?;

        // The default behavior of terminal::Clear is to maintain the cursor position
        // If the user wants to reset the cursor position, it needs to be done manually
        if !keep_cursor_pos {
            queue!(stdout(), cursor::MoveTo(0, 0))?;
        }

        if direct_execute {
            stdout().flush()?;
        }

        Ok(())
    }

    // [Direct] Performs a frame update, clearing the screen and redrawing the buffer
    fn update(&self, reset_cursor: bool) -> Result<()> {
        // Clear the screen
        self.clear_screen(!reset_cursor, false)?;

        // Save the position of the cursor
        // This could be be either the position of the cursor at the start of the frame update,
        // Or the (0, 0) position if the cursor is being reset
        execute!(stdout(), cursor::SavePosition)?;

        // Draw the buffer, making sure to carriage return after each line
        for line in self.buffer.lines() {
            queue!(stdout(), Print(line), Print("\r"))?;
        }

        // Restore the cursor position to its saved state
        if reset_cursor {
            queue!(stdout(), cursor::RestorePosition)?;
        }

        stdout().flush()
    }

    // [Direct] Closes the terminal and exits the program
    #[allow(dead_code)]
    fn exit(&self) -> Result<()> {
        // Disable raw mode so the terminal can be used normally
        terminal::disable_raw_mode()?;

        // Clear the screen
        self.clear_screen(false, true)?;

        // Exit the program
        std::process::exit(0);
    }

    // Gets the cursor position in relation to the buffer rather than the terminal
    #[allow(dead_code)]
    fn get_rope_coordinate(&self) -> Result<usize> {
        let (cursor_x, cursor_y) = cursor::position()?;
        Ok((cursor_y as usize) * self.window_length + cursor_x as usize)
    }
}
