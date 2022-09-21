pragma solidity 0.5.12;


interface IDEP{
	function nNodeUnSpecifiedAddressTask(
	        string calldata url, 
	        string  calldata options, 
	        uint64 maxRunNum, 
	        uint64 maintainBlocks
	    ) external;
    function blockUintPrice() external returns(uint64);
	function proofUnit() external returns(uint256);
}

interface IEZC{
	function transferFrom(address from, address to, uint256 amount) external;
}

contract WorkCenter {
	struct Work{
		string url;
		string options;
	}

	address public owner;
	IEZC public ezc;
	IDEP public dep;
	Work public currentWork;


	event NewWork(string url, uint256 time);

	modifier onlyOwner{
		require(msg.sender == owner);
        _;
	}	

	constructor(address _ezc, address _dep) public {
		owner = msg.sender;
		ezc = IEZC(_ezc);
		dep = IDEP(_dep);
	}

	function setWork(string calldata workURL, string calldata options) external onlyOwner{
		currentWork.url = workURL;
		currentWork.options = options;
		emit NewWork(workURL, block.timestamp);
	}

	function publishWork(uint64 maxRunNum, uint64 maintainBlocks) external{
		uint256 proof = calcProof(maxRunNum, maintainBlocks);
		ezc.transferFrom(msg.sender, address(this), proof);
		dep.nNodeUnSpecifiedAddressTask(currentWork.url, currentWork.options, maxRunNum, maintainBlocks);
	}

	function calcProof(uint64 maxRunNum, uint64 maintainBlocks) internal returns(uint256){
		uint256 taskTotalPrice = 0;
        uint64 blockPrice = 1;

        if (maintainBlocks > 100) blockPrice = maintainBlocks / dep.blockUintPrice();
        
        return taskTotalPrice = dep.proofUnit() * maxRunNum * blockPrice;
	}

}