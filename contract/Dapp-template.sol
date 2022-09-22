//SPDX-License-Identifier: Unlicense
pragma solidity ^0.8.0;

interface IDEP{
	function nNodespecifiedAddressTask(
	        string calldata url, 
			string calldata options, 
			uint64 maxRunNum, 
			address[] memory receivers, 
			uint64 maintainBlocks
	    ) external;
    function blockUintPrice() external returns(uint64);
	function proofUnit() external returns(uint256);
}

contract DepTaskBridge {
	struct Task{
		string url;
		string options;
	}

	address public owner;
	IDEP public dep;
	Task public currentWork;


	event NewTaskChange(string url, uint256 time);

	modifier onlyOwner{
		require(msg.sender == owner);
        _;
	}	

	constructor(address _dep) {
		owner = msg.sender;
		dep = IDEP(_dep);
	}

	function setDEP(IDEP _dep) external onlyOwner {
		dep = _dep;
    }

	function setTask(string calldata workURL, string calldata options) external onlyOwner{
		currentWork.url = workURL;
		currentWork.options = options;
		emit NewTaskChange(workURL, block.timestamp);
	}

	function payForTask(address[] memory user_address, uint64 maintainBlocks) external{
		dep.nNodespecifiedAddressTask(currentWork.url, currentWork.options, 1, user_address, maintainBlocks);
	}
}