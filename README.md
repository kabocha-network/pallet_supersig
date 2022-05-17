# Supersig Pallet

The supersig pallet is a multisig with super powers.
It allows you to add and remove members of the multisig.
It extends the capabilities of a multisig so it can be fit for governance of larger funds.

A multisig transaction acts more like a funding proposal.
And the signatures become votes, with a quorum that can be changed

Good to know: the multisig addresses won’t change even though the members can be added or removed.

## Overview

The Supersig pallet provide function for:

- Creating a supersig
- Submit proposal to a supersig
- Vote the proposal
- Remove a current proposal


### Dispatchable Functions

- `create_supersig` - create a supersig, with specified members and threshold
- `submit_call` - make a proposal on the specified supersig
- `approve_call` - give a positive vote to a call. if the number of vote = threshold, the call
is executed
- `remove_call` - remove a call from the poll

