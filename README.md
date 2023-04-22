<h1 align="center">
 
 <a href="https://decentration.org"> <img src="./assets/supersig.jpg" alt="Supersig" width="500"></a>

</h1>

_Funded by Web3 foundation and Edgeware_
<h2 align="center">
<a href="https://decentration.org"> <img img src="./assets/contributors.png" alt="Rusty Crewmates" margin="10px" width="200"></a>

</h2>


# Supersig Pallet

The supersig pallet extends the capabilities of a multisig so it can be fit for governance of
larger funds. It is a superset of the multisig pallet, adding multiple functionalities and
options to the original multi-signature dispatch allowing multiple signed origins (accounts) to
coordinate and dispatch a call from the supersig account

Note: the multisig addresses won’t change even though the members can be added, removed, or can
leave themselves

## Overview

The Supersig pallet provide function for:

- Creating a supersig
- Adding and removing members
- Leaving the supersig
- Submit transaction to a supersig
- Vote for the transaction
- Remove a pending transaction
- Delete a supersig


### Dispatchable Functions

- `create_supersig` - create a supersig, with specified members. The creator will have to
  deposit an existencial balance and a deposit that depend on the number of members, in the
  supersig account. This last amount will be reserved on the supersig

  /!!\ note of caution /!!\ the creator of the supersig will NOT be added by default, he will
  have to pass his adress into the list of added users.

- `propose_call` - make a proposal on the specified supersig. an amount corresponding to the
  length of the encoded call will be reserved.

- `approve_call` - give a positive vote to a call. if the number of vote >= SimpleMajority, the
  call is executed. An user can only approve a call once.

- `remove_call` - remove a call from the poll. The reserved amount of the proposer will be
  unreserved

- `add_members` - add new members to the supersig. In case some user are already in the
  supersig, they will be ignored.

- `remove_members` - remove members from the supersig. In case some user are not in the
  supersig, they will be ignored.

- `remove_supersig` - remove the supersig and all the associated data. Funds will be unreserved
  and transfered to specified beneficiary.

- `leave_supersig` - remove the caller from the supersig.

## Test

To run the tests in this pallet run:
`cargo test`



### Licence

SPDX-License-Identifier: Apache-2.0

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

