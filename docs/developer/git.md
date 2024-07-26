# Git push

## Clean current branch

On your developer branch, discard all the messy modifications (make sure your useful changes have already been committed).

```
git stash && git stash clear
```

## Checkout master branch

```
git checkout master
git pull
```

## Merge master to your current branch

```
git checkout dev-branch
git rebase master
```

Then, the system will prompt you to manually merge the differences.

## Push branch

```
git push origin dev-branch
```

Then your can submit a pull request to merge.