# Stores

cake keeps each item as a markdown file in a store. A store is always a `.cake` directory. cake looks for one in this order: the nearest `.cake` in the current directory or above it, then a directory linked in `~/.config/cake/stores.json`, then the global store in `~/.local/share/cake/.cake`.

Wherever a command takes a store (`--store`, `CAKE_STORE`, `cake mv --to-store`), you can name the directory or its `.cake`. `~/git/work` and `~/git/work/.cake` mean the same store. `cake mv --to-store` also takes the name a store has in `stores.json`, or `global`.

## A project store

Inside a code repo, `cake init` creates the store as a worktree on its own branch `cake/main`. Items never mix with your code commits, and `.cake` is excluded from the code branch.

```sh
cd ~/git/billing
cake init
cake sync --publish
```

If the branch already exists locally or on the remote, `cake init` attaches to it instead of starting a new one. Outside a git repo, `cake init` creates a plain `.cake` without commits. Run `cake init` again after `git init` and the items move into the new branch.

## A work store used from several repos

Make a repo for your work todos and create the store in it. In a git repo the store is a worktree on its own branch `cake/main`, so `main` can hold other files.

```sh
git clone git@example.org:me/work.git ~/git/work
cd ~/git/work
cake init
cake sync --publish
```

Give each project a goal. Goals are notes tagged `goal` and can nest.

```sh
cake add work --note -t goal -m "# Work"
cake add billing --note -t goal -p work -m "# Billing service"
cake add reports --note -t goal -p work -m "# Quarterly reports"
```

## Linking a repo to a goal

In each repo you work in, link it to the store and name its goal.

```sh
cd ~/git/billing
cake --store ~/git/work init --parent billing
```

From then on, in that repo and every directory below it:

- `cake add` without `-p` puts the item under `billing`. `-p -` puts it at the top level, `-p <id>` under another item.
- `cake ls`, `cake ready` and the form show only the `billing` subtree. `cake ls --wide` shows the whole store.
- `-g` and `--store` ignore the goal.

Run the same command with another `--parent` to change the goal. Leave out `--parent` to link without a goal. In `~/git/work` itself you always see the whole store.

The link lives in `~/.config/cake/stores.json`:

```json
{
  "work": {
    "store": "/home/me/git/work/.cake",
    "dirs": [
      { "path": "/home/me/git/billing", "parent": "billing" },
      "/home/me/notes"
    ]
  }
}
```

The deepest linked directory wins. A `.cake` in the directory tree always wins over a link. If the goal is renamed with `cake rename`, the link follows. If it is removed, items land at the top level with a warning.

`cake --store <dir> init` never creates a store. If `<dir>` has none, it fails and tells you to run `cake init` there first.

## The global store

The global store holds personal things that belong to no project. Use it from anywhere with `-g`.

```sh
cake -g add -m "# Renew passport"
```

It is created as a plain `.cake` on first use. To sync it, make it a git repo and publish it. `cake -g init` turns `~/.local/share/cake` into a git repo and moves the existing items into a worktree on `cake/main`.

```sh
cake -g init
git -C ~/.local/share/cake remote add origin git@example.org:me/personal.git
cake -g sync --publish
```

## Another machine

Clone the repo of the store and run `cake init`. It finds the branch on the remote and attaches to it.

```sh
git clone git@example.org:me/work.git ~/git/work
cd ~/git/work
cake init
```

Links are per machine, so link each repo again.

```sh
cd ~/git/billing
cake --store ~/git/work init --parent billing
```

For the global store, clone it to `~/.local/share/cake` and run `cake -g init`.

## Syncing

`cake sync` fetches, merges and pushes the store you are in. Run it from the store's directory or any linked directory. Every change through cake is already committed, so there is nothing to stage.

```sh
cake sync
cake -g sync
cake --store ~/git/work sync
```

After a conflict, fix the item files and run `cake sync --continue`, or give up with `cake sync --abort`.

## Old layouts

Older versions kept the global store, and stores made with `cake --store <dir> init`, directly in their directory with `items/` at the top. cake moves such a store into `<dir>/.cake` the first time it uses it, git repo included, and says so once. Only the global directory and stores listed in `stores.json` move. Any other directory with an `items/` folder is left alone.
