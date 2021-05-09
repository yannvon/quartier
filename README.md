# Quartier

A decentralized application that connects a neighborhood, creating a place where good deeds can be rewarded, issues discussed and votes be held.

This project came to life during the *Scaling Ethereum* hackathon 2021.

The goal is to leverage the privacy properties of secret contracts on the Secret Network to create the required decentralized back-end.

<img src="pics\4.jpg" alt="1" style="zoom:80%;" />

## Voting on the Secret Network

I will start by writing the core voting mechanism, that is kept very general for now.

### Some e-voting theory

The main security objectives for any e-voting scheme are the following:

1. **Accuracy**: The result reflects the choice of the voters

 2. **Secrecy**: The vote of each voter remains secret
 3. **No provisional results**: There is no information about provisional results during the election

The typical risks pertaining to these factors are the following:

1. Accuracy 
   1. Double votes (e.g. over two channels)
   2. Manipulation of votes (e.g. on the voters machine while voting during transmission over Internet, by hacking servers
   3. Fake votes, given without authorization (voting card)
2. Secrecy
   1. Interception of votes (e.g. on the voters machine while voting, during transmission over Internet, by hacking servers)
3. No provisional results:
   1. Interception of votes (e.g. on the voters machine while voting, during transmission over Internet, by hacking servers)

 

Additionally this voting scheme can explore many desirable properties that are hard to implement
 in other technologies, some of these properties are:

1. **Individual verifiability**: An individual has proof that their vote has been correctly taken into account-protects against a man-in-the-browser that changes outgoing votes and incoming confirmation (you think you voted ‘yes’ but you voted ‘no’
2. **Universal verifiability**: We have proof that all votes have been correctly counted-protects against attacks on the server, that delete, add or modify some votes[^1]

### Other desirable features

- [x] Set time frame in which people can respond to vote
- [ ] Restrict vote to subset of entities/addresses.
- [ ] Add  support for mini publics (only randomly selected addresses can vote, more on that below)
- [x] Individual verifiability, (ideally in a way that does not allow to sell your vote
- [ ] Liquid democracy (allowing someone else to cast a vote for you, i.e. voting the same as them)
- [x] One can change its mind, and change its vote, as long as the vote is still ongoing. (removed in newest version as it conflicts with liquid democracy)



### How the contract works

**Disclaimer**: The following template [ 2 ] was used to create the contract. The basic structure of the contract is taken from [ 3 ] and is also inspired by [ 4 ].





### Useful links

- https://learn.figment.io/network-documentation/secret/tutorials/creating-a-secret-contract-from-scratch
- https://github.com/enigmampc/secret-contracts-guide
- https://github.com/enigmampc/secret-toolkit

- https://learn.figment.io/network-documentation/secret

- https://github.com/enigmampc/SecretJS-Templates

- https://build.scrt.network/dev/quickstart.html#create-initial-smart-contract






[^1]: The entire section was taken from "Crypto for e-voting protocols", a lecture given by Prof. Oechslin at EPFL in 2019.
[^2]: Image taken from https://commons.wikimedia.org/wiki/File:Lombard_Street,_San_Francisco._(Unsplash).jpg

https://pixabay.com/illustrations/ballot-box-vote-ballot-box-icon-5676561/



[2]: https://github.com/enigmampc/secret-template
[3]: https://github.com/enigmampc/SecretSimpleVote
[4]: https://github.com/baedrik/SCRT-sealed-bid-auction/blob/master/src/contract.rs

