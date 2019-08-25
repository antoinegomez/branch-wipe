# branch-wipe git utility

I work daily with git and often end up with a lot of branches on my projects that I don't delete because I am simple to lazy to run `git branch -D branch_name` once it is merged.

I have done it to learn more about rust and gui application.
<br>All of the code was copied from one example of gtk-rs examples repository and tweaked to do what I wanted.

[gtk-rs examples](https://github.com/gtk-rs/examples)


Provided as it. There are stuff to improve even for a little app like this. If you have any remarks feel free to open a pull request: I am here to learn.


# Usage

Build the app:

`cargo build --release --features="gtk/v3_16 gio/v2_44 subclassing"`

Move the generated binary into one directory of your $PATH variable.

Launch it and you shall see a list of branches and a delete button.


# Screenshot

[screenshot](https://i.imgur.com/A4Frmww.png)


