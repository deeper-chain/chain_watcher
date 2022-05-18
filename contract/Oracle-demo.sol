//SPDX-License-Identifier: Unlicense
pragma solidity ^0.8.0;

contract Oracle {
  // Contract owner
    address public owner;

    // Token MarketPrice Storage
    uint public tokenMarketPrice;

    // Callback function
    event CallbackGetTokenPrice();

    constructor() {
        owner = msg.sender;
    }

    function setTokenPrice(uint feedPrice) public {
        // If it isn't sent by a trusted oracle
        // a.k.a ourselves, ignore it
        require(msg.sender == owner);
        tokenMarketPrice = feedPrice;
        emit CallbackGetTokenPrice();
    }

    function getTokenPrice() view public returns (uint) {
        return tokenMarketPrice;
    }
}