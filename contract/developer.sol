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


interface IERC20 {
    function transferFrom(address from, address to, uint256 amount) external;
}

contract DepTaskBridge {
	struct Task{
		string url;
		string options;
	}

	address public owner;
	IDEP public dep;
	Task public currentWork;
    IERC20 ezc;


	event NewTaskChange(string url, uint256 time);

	modifier onlyOwner{
		require(msg.sender == owner);
        _;
	}	

	constructor(address _dep, address _ezc) {
		owner = msg.sender;
		dep = IDEP(_dep);
        IERC20 ezc = IERC20(_ezc);
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
        uint256 totalPrice = calcEZC(1, maintainBlocks);
        ezc.transferFrom(msg.sender, address(this),totalPrice);
		dep.nNodespecifiedAddressTask(currentWork.url, currentWork.options, 1, user_address, maintainBlocks);
	}

	function calcEZC(uint256 maxRunNum, uint64 maintainBlocks) internal returns (uint256){
		uint64  blockUintPrice = 100;
		uint256 proofUnit = 1 ether;
		uint256 taskTotalPrice = 0;
        uint64 blockPrice = 1;
		if (maintainBlocks > 100) blockPrice = maintainBlocks / blockUintPrice;
        
        taskTotalPrice = proofUnit * maxRunNum * blockPrice;
	}
}