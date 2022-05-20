//SPDX-License-Identifier: Unlicense
pragma solidity ^0.8.0;

interface IEZC {
    /**
     * @dev Returns the amount of tokens in existence.
     */
    function totalSupply() external view returns (uint256);

    /**
     * @dev Returns the amount of tokens owned by `account`.
     */
    function balanceOf(address account) external view returns (uint256);

    /**
     * @dev Moves `amount` tokens from the caller's account to `to`.
     *
     * Returns a boolean value indicating whether the operation succeeded.
     *
     * Emits a {Transfer} event.
     */
    function transfer(address to, uint256 amount) external returns (bool);

    /**
     * @dev Returns the remaining number of tokens that `spender` will be
     * allowed to spend on behalf of `owner` through {transferFrom}. This is
     * zero by default.
     *
     * This value changes when {approve} or {transferFrom} are called.
     */
    function allowance(address owner, address spender) external view returns (uint256);

    /**
     * @dev Sets `amount` as the allowance of `spender` over the caller's tokens.
     *
     * Returns a boolean value indicating whether the operation succeeded.
     *
     * IMPORTANT: Beware that changing an allowance with this method brings the risk
     * that someone may use both the old and the new allowance by unfortunate
     * transaction ordering. One possible solution to mitigate this race
     * condition is to first reduce the spender's allowance to 0 and set the
     * desired value afterwards:
     * https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729
     *
     * Emits an {Approval} event.
     */
    function approve(address spender, uint256 amount) external returns (bool);

    /**
     * @dev Moves `amount` tokens from `from` to `to` using the
     * allowance mechanism. `amount` is then deducted from the caller's
     * allowance.
     *
     * Returns a boolean value indicating whether the operation succeeded.
     *
     * Emits a {Transfer} event.
     */
    function transferFrom(
        address from,
        address to,
        uint256 amount
    ) external returns (bool);

    /**
     * @dev Emitted when `value` tokens are moved from one account (`from`) to
     * another (`to`).
     *
     * Note that `value` may be zero.
     */
    event Transfer(address indexed from, address indexed to, uint256 value);

    /**
     * @dev Emitted when the allowance of a `spender` for an `owner` is set by
     * a call to {approve}. `value` is the new allowance.
     */
    event Approval(address indexed owner, address indexed spender, uint256 value);

    /**
     * @dev Destroys `amount` tokens from the caller.
     *
     * See {ERC20-_burn}.
     */
    function burn(uint256 amount) external;

    /**
     * @dev Destroys `amount` tokens from `account`, deducting from the caller's
     * allowance.
     *
     * See {ERC20-_burn} and {ERC20-allowance}.
     *
     * Requirements:
     *
     * - the caller must have allowance for ``accounts``'s tokens of at least
     * `amount`.
     */
    function burnFrom(address account, uint256 amount) external;
}

contract DeeperMachine {
    struct Task {
        uint64 currentRunNum;
        uint64 maxRunNum;
        uint64 startTime;
        address[] receivers;
    }

    mapping(address => mapping(uint64 => bool)) public userTask;
    mapping(address => mapping(uint64 => bool)) public userTaskCompleted;

    mapping(uint64 => Task) public taskInfo;

    mapping(address => mapping(uint64 => uint64)) public userDayIncome;
    mapping(uint64 => uint64) public dayTotalIncome;

    mapping(address => uint64) public userSettledDay;

    event TaskPublished(uint64 taskId, string url, string options, uint64 maxRunNum, address[] receivers);
    event RaceTask(address node, uint64 taskId);
    event ResetRunners(address[] receivers);

    uint64 public taskSum = 0;
    address public owner;

    uint64 public price = 1 ether;
    uint64 public raceTimeout = 20 minutes;
    uint64 public completeTimeout = 48 hours;
    uint64 public startDay;

    IEZC ezc;
    constructor(IEZC _ezc) {
        owner = msg.sender;
        startDay = uint64(block.timestamp / 1 days);

        ezc = _ezc;
    }

    modifier onlyOwner {
        require(msg.sender == owner, "not owner address");
        _;
    }

    function implementationVersion() external pure virtual returns (string memory) {
        return "1.0.1";
    }

    function setEZC(IEZC _ezc) external onlyOwner {
        ezc = _ezc;
    }

    function setPrice(uint64 _price) external onlyOwner {
        price = _price;
    }

    function setRaceTimeout(uint64 _raceTimeout) external onlyOwner {
        raceTimeout = _raceTimeout;
    }

    function setCompleteTimeout(uint64 _completeTimeout) external onlyOwner {
        completeTimeout = _completeTimeout;
    }

    function publishTask(string calldata url, string calldata options, uint64 maxRunNum, address[] memory receivers) external payable {
        ezc.burnFrom(msg.sender, price * maxRunNum);
        uint64 day = uint64(block.timestamp / 1 days);
        dayTotalIncome[day] += price * maxRunNum;
        taskSum = taskSum + 1;
        taskInfo[taskSum].maxRunNum = maxRunNum;
        taskInfo[taskSum].currentRunNum = 0;
        taskInfo[taskSum].startTime = uint64(block.timestamp);
        taskInfo[taskSum].receivers = receivers;

        emit TaskPublished(taskSum, url, options, maxRunNum, receivers);
    }

    function resetRunners(address[] memory receivers) external {
        emit ResetRunners(receivers);
    }

    function raceSubIndexForTask(uint64 taskId) external {
        require(taskSum >= taskId, "Invalid taskId");
        require(taskInfo[taskId].maxRunNum >= taskInfo[taskId].currentRunNum + 1, "Task has been filled");
        require(taskInfo[taskId].startTime + raceTimeout >= block.timestamp, "Task race has been expired");

        if (taskInfo[taskId].receivers.length > 0) {
            bool exists = false;
            for (uint i = 0; i < taskInfo[taskId].receivers.length; i++) {
                if (taskInfo[taskId].receivers[i] == msg.sender) {
                    exists = true;
                    break;
                }
            }
            require(exists, "Invalid task receiver");
        }

        require(!readSubIndexForTask(taskId), "Address already used");

        userTask[msg.sender][taskId] = true;
        taskInfo[taskId].currentRunNum = taskInfo[taskId].currentRunNum + 1;

        emit RaceTask(msg.sender, taskId);
    }

    function completeSubIndexForTask(uint64 taskId) external {
        require(userTask[msg.sender][taskId], "Invalid taskId or task not raced");
        require(!userTaskCompleted[msg.sender][taskId], "Sub task has been completed");
        require(taskInfo[taskId].startTime + completeTimeout >= block.timestamp, "Task has been expired");

        userTaskCompleted[msg.sender][taskId] = true;

        uint64 day = uint64(block.timestamp / 1 days);
        userDayIncome[msg.sender][day] += price;
    }

    function getUserIncomeForDay(address user, uint64 theDay) public view returns (uint64){
        return userDayIncome[user][theDay];
    }

    function getMyIncomeForDay(uint64 theDay) public view returns (uint64){
        return getUserIncomeForDay(msg.sender, theDay);
    }

    function getUserContributionForDay(address user, uint64 theDay) public view returns (uint64){
        return getUserIncomeForDay(user, theDay) * 10000 / getTotalIncomeForDay(theDay);
    }

    function getMyContributionForDay(uint64 theDay) public view returns (uint64){
        return getUserContributionForDay(msg.sender, theDay);
    }

    function getTotalIncomeForDay(uint64 theDay) public view returns (uint64){
        return dayTotalIncome[theDay];
    }

    function readSubIndexForTask(uint64 taskId) public view returns (bool) {
        return userTask[msg.sender][taskId];
    }

    function withdrawFund() external onlyOwner {
        address payable powner = payable(owner);
        powner.transfer(address(this).balance);
    }
}