use traverse_lsp::traverse_adapter::TraverseAdapter;

const SIMPLE_CONTRACT: &str = r#"
pragma solidity ^0.8.0;

contract SimpleToken {
    mapping(address => uint256) private balances;
    uint256 public totalSupply;
    
    event Transfer(address indexed from, address indexed to, uint256 value);
    
    constructor(uint256 _initialSupply) {
        totalSupply = _initialSupply;
        balances[msg.sender] = _initialSupply;
        emit Transfer(address(0), msg.sender, _initialSupply);
    }
    
    function transfer(address to, uint256 amount) public returns (bool) {
        require(balances[msg.sender] >= amount, "Insufficient balance");
        balances[msg.sender] -= amount;
        balances[to] += amount;
        emit Transfer(msg.sender, to, amount);
        return true;
    }
    
    function balanceOf(address account) public view returns (uint256) {
        return balances[account];
    }
}
"#;

const COMPLEX_CONTRACT: &str = r#"
pragma solidity ^0.8.0;

interface IERC20 {
    function transfer(address to, uint256 amount) external returns (bool);
    function balanceOf(address account) external view returns (uint256);
}

contract DeFiVault {
    IERC20 public token;
    mapping(address => uint256) public deposits;
    
    event Deposit(address indexed user, uint256 amount);
    event Withdrawal(address indexed user, uint256 amount);
    
    constructor(address _token) {
        token = IERC20(_token);
    }
    
    function deposit(uint256 amount) external {
        require(token.transferFrom(msg.sender, address(this), amount), "Transfer failed");
        deposits[msg.sender] += amount;
        emit Deposit(msg.sender, amount);
    }
    
    function withdraw(uint256 amount) external {
        require(deposits[msg.sender] >= amount, "Insufficient deposit");
        deposits[msg.sender] -= amount;
        require(token.transfer(msg.sender, amount), "Transfer failed");
        emit Withdrawal(msg.sender, amount);
    }
    
    function getBalance() external view returns (uint256) {
        return token.balanceOf(address(this));
    }
}
"#;

#[test]
fn test_workspace_call_graph() {
    let adapter = TraverseAdapter::new().expect("Failed to create adapter");
    let graph = adapter
        .build_call_graph(SIMPLE_CONTRACT)
        .expect("Failed to build call graph");

    assert!(graph.nodes.len() > 0);
    assert!(graph.edges.len() > 0);

    let has_constructor = graph.nodes.iter().any(|n| n.name == "SimpleToken");
    let has_transfer = graph.nodes.iter().any(|n| n.name == "transfer");
    let has_balance = graph.nodes.iter().any(|n| n.name == "balanceOf");

    assert!(has_constructor);
    assert!(has_transfer);
    assert!(has_balance);
}

#[test]
fn test_workspace_dot_generation() {
    let adapter = TraverseAdapter::new().expect("Failed to create adapter");
    let graph = adapter
        .build_call_graph(COMPLEX_CONTRACT)
        .expect("Failed to build call graph");
    let dot = adapter
        .generate_dot_diagram(&graph)
        .expect("Failed to generate DOT");

    assert!(dot.contains("digraph"));
    assert!(dot.contains("DeFiVault"));
    assert!(dot.contains("deposit"));
    assert!(dot.contains("withdraw"));
    assert!(dot.contains("->"));
}

#[test]
fn test_workspace_mermaid_generation() {
    let adapter = TraverseAdapter::new().expect("Failed to create adapter");
    let graph = adapter
        .build_call_graph(SIMPLE_CONTRACT)
        .expect("Failed to build call graph");
    let mermaid = adapter
        .generate_mermaid_flowchart(&graph)
        .expect("Failed to generate Mermaid");

    assert!(mermaid.contains("sequenceDiagram"));
    assert!(mermaid.contains("SimpleToken"));
    assert!(mermaid.contains("transfer"));
}
