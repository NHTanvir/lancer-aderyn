// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0; // unspecific-solidity-pragma

// A contrived contract exercising several lancer-aderyn detectors. Run:
//   lancer-aderyn tests/contract-playground/Vulnerable.sol --stdout
// (requires `solc` on PATH).

interface IERC20 {
    function transfer(address to, uint256 amount) external returns (bool);
    function transferFrom(address from, address to, uint256 amount) external returns (bool);
}

contract Vulnerable {
    address public owner;
    uint256 public lastSeed;
    mapping(address => uint256) public balances;

    constructor() {
        owner = msg.sender;
    }

    // tx-origin-used-for-auth
    function onlyOriginAuth() external view returns (bool) {
        return tx.origin == owner;
    }

    // weak-randomness
    function badRandom() external view returns (uint256) {
        return uint256(keccak256(abi.encodePacked(block.timestamp, block.prevrandao)));
    }

    // unchecked-low-level-call
    function pay(address payable to) external {
        to.call{value: 1 ether}(""); // result ignored
    }

    // arbitrary-transfer-from
    function pull(IERC20 token, address from, uint256 amount) external {
        token.transferFrom(from, msg.sender, amount);
    }

    // selfdestruct
    function destroy() external {
        selfdestruct(payable(owner));
    }

    // todo
    function refactorMe() external {
        // TODO: add access control here
        balances[msg.sender] += 1;
    }

    // empty-block
    function emptyOne() external {}
}
