//SPDX-License-Identifier: Unlicense
pragma solidity ^0.8.0;

contract Oracle {
    uint public dotMarketPrice;
    event CallbackGetTokenPrice(uint currentTokenPrice);

    function setTokenPrice(uint feedPrice) public {
        dotMarketPrice = feedPrice;
        emit CallbackGetTokenPrice(dotMarketPrice);
    }
}