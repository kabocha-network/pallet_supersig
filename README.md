<h1 align="center">
 
 <a href="https://decentration.org"> <img src="./assets/supersig.jpg" alt="Supersig" width="500"></a>

</h1>

_Key Contributor: Rusty Crewmates; Funded by Web3 foundation and Edgeware._
<h2 align="center">
<a href="https://decentration.org"> <img img src="./assets/contributors.png" alt="Rusty Crewmates" margin="10px" width="200"></a>

</h2>


# Supersig Pallet

This pallet provides functionality for creating and managing supersigs, which is designed to be 
more flexible than multisig, but with some trade-offs. 

A supersig allow a group of members to collectively make decisions on behalf of an on-chain entity. Each member is assigned
a role, either "Master" or "Standard", which determines their voting power in the decision-making
process.

The supersig pallet extends the capabilities of a multisig so it can be fit for governance of
larger funds. It is a superset of the multisig pallet, adding multiple functionalities and
options to the original multi-signature dispatch allowing multiple signed origins (accounts) to
coordinate and dispatch a call from the supersig account

## Overview
The Supersig pallet provide function for:
- Creating a supersig organisation
- Adding and removing members
- Leaving the supersig
- Submit a proposal to execute a transaction
- Vote for the transaction
- Remove a pending transaction
- Delete a supersig

### Dispatchable Functions

- `create_supersig` - Create a supersig, with specified members. The creator will have to
  deposit an existencial balance and a deposit that depend on the number of members, in the
  supersig account. This last amount will be reserved on the supersig
  /!!\ Reminder /!!\ the creator of the supersig will NOT be added by default, he will
  have to pass his address into the list of added users.

- `propose_call` - Submit a proposal for the supersig to execute a transaction, which is an amount corresponding to the
  length of the encoded call will be reserved. The call wraps around the extrinsic which the user is proposing to execute.
   (Anything that requires a vote needs to be wrapped in a proposeCall function).

- `approve_call` - Vote for the call to be execute. The threshold is enumerated to vote >= SimpleMajority, the
  call is executed. A user can only approve a call once.

- `remove_call` - Remove a call from the poll. The reserved amount of the proposer will be unreserved.

- `add_members` - Add new members to the supersig organisation. In case some user are already in the
  supersig, they will be ignored.

- `remove_members` - Remove members from the supersig. 

- `delete_supersig` - Remove the supersig and all the associated data. Funds will be unreserved
  and transfered to specified beneficiary.

- `leave_supersig` - Elect to leave the supersig. You cannot leave if you are the last member, instead you would
   need to `delete_supersig`.

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

