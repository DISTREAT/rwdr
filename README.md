# RWDR

RWDR reads all packages on an Arch Linux system,
filtering for files that are not recoverable by Pacman.

## Concept

MTrees, created by the Arch Linux package manager (`/var/lib/pacman/local/*/mtree`),
keep track of files included in packages. Therefore, the respective
checksums within MTree files may be compared against the root file system.

Using the checksums we can identify files that are not recoverable by Pacman -
or in simple terms - we get the file system minus all installed software.

Although originally developed for backup purposes, I cannot recommend using
this tool to create a backup of your system. Nonetheless, this tool should
yield files that you might have placed deep into the file system and have
forgotten about.

At the moment `/etc`, `/usr`, `/boot`, `/opt`, `/srv`, and `/var` are used for comparison.
