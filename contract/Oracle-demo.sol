//SPDX-License-Identifier: Unlicense
pragma solidity ^0.8.0;

contract Oracle {
  // Contract owner
    address public owner;

    // Token MarketPrice Storage
    uint public tokenMarketPrice;

    // Callback function
    event CallbackGetTokenPrice(uint currentTokenPrice);

    constructor() {
        owner = msg.sender;
    }

    function setTokenPrice(uint feedPrice) public {
        tokenMarketPrice = feedPrice;
        emit CallbackGetTokenPrice(tokenMarketPrice);
    }

    function getTokenPrice() view public returns (uint) {
        return tokenMarketPrice;
    }
}