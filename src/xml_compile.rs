use std::collections::HashMap;

// 检测是否是空白, 经检测正则表达式性能过低，这里不需要复杂的匹配，直接等于判断
pub fn is_white_space(target: char) -> bool {
    if target == ' ' || target == '\r' || target == '\n' || target == '\t' {
        true
    } else {
        false
    }
}

// 删除两侧多余的空白
pub fn trim_string(string: &[char]) -> Option<&[char]> {
    let len = string.len();
    let mut head = 0;
    let mut tail = len - 1;
    while head < len {
        // 如果是空白，头部后移
        if is_white_space(string[head]) {
            head += 1;
        } else {
            // 不是空白，退出循环
            break;
        }
    }
    while tail > 0 as usize {
        // 如果是空白，尾部前移
        if is_white_space(string[tail]) {
            tail -= 1;
        } else {
            // 不是空白，退出循环
            break;
        }
    }
    // 尾部大于等于头部，才有有效字符
    if tail >= head {
        // 截取时尾部需要加一，否则最后一个截取不到
        return Some(&string[head..tail + 1]);
    }
    // 尾部小于头部，该字符串全是空，返回一个空
    None
}

// props 中的值
#[derive(Debug)]
pub enum PropValue<'a> {
    StringValue(&'a [char]),
    BooleanValue(bool),
}

// props
type Props<'a> = HashMap<&'a [char], PropValue<'a>>;

// expressions
type Expressions<'a> = Vec<&'a [char]>;

// children
type Children<'a> = Vec<XMLTreeNode<'a>>;

#[derive(Debug)]
pub struct Tag<'a> {
    pub node_name: &'a [char],
    // 已解析的属性
    pub props: Option<Props<'a>>,
    // 是否是单标签
    pub is_single_tag: bool,
    pub children: Option<Children<'a>>,
}

#[derive(Debug)]
pub struct TagEnd<'a> {
    pub node_name: &'a [char],
}

// 注释节点
#[derive(Debug)]
pub struct Comment<'a> {
    value: &'a [char],
}

// 文档碎片
#[derive(Debug)]
pub struct Fragment<'a> {
    children: Option<Children<'a>>,
}

// 文档碎片
#[derive(Debug)]
pub struct FragmentEnd {}

// 节点
#[derive(Debug)]
pub enum Node<'a> {
    Tag(Tag<'a>),
    TagEnd(TagEnd<'a>),
    Fragment(Fragment<'a>),
    FragmentEnd(FragmentEnd),
    Comment(Comment<'a>),
    String(&'a [char]),
}

pub type Nodes<'a> = Vec<Node<'a>>;

// 最后解析出的XMLTreeNode, 不包含结束节点
#[derive(Debug)]
pub enum XMLTreeNode<'a> {
    Tag(Tag<'a>),
    Fragment(Fragment<'a>),
    Comment(Comment<'a>),
    String(&'a [char]),
}

pub fn parse_to_nodes<'a>(xml: &'a Vec<char>) -> Nodes<'a> {
    // 遍历索引
    let mut cursor: usize = 0;
    // 当前字符
    let mut current_char;
    // 解析出的节点
    let mut nodes: Nodes = Vec::new();
    // 长度
    let length = xml.len();

    // 标签之外全部视为string
    let mut string_start_index = cursor;

    while cursor < length {
        // 当前字符
        current_char = xml[cursor];
        if current_char == '<' {
            // 是否是注释节点,如果是，则最后的时候对node不在做动作
            let mut is_comment_node = false;

            // 发现括号就暂停收集 string
            // 结束大于开始+1就可以收集字符串（否则字符为空）
            if cursor > string_start_index {
                // 清理两侧空白
                if let Some(string) = trim_string(&xml[string_start_index..cursor]) {
                    nodes.push(Node::String(string));
                }
            }

            // 解析标签
            let mut node: Option<Node> = None;

            // 节点名
            let mut node_name: &[char] = &[];

            // 表达式
            let mut expressions: Expressions = Vec::new();

            // node_name 是否匹配结束
            let mut node_name_match_end = false;
            // 记录括号内的内容开始索引
            let start_index = cursor + 1;
            // 解析，得到一个节点
            // 记录表达式开始索引, 括号中的每个项都看作一个表达式,这里初始化完全没用，只是为了rust检查，初始化在下方node_name收集完毕后
            let mut expression_start_index: usize = cursor + 1;
            while current_char != '>' {
                cursor += 1;
                current_char = xml[cursor];

                if current_char == '\"' || current_char == '\'' {
                    // 字符串可能是 " 或 ' 包裹
                    let string_pattern = current_char;
                    // 跳过字符解析
                    cursor += 1;
                    current_char = xml[cursor];
                    while current_char != string_pattern {
                        cursor += 1;
                        current_char = xml[cursor];
                    }
                }

                // 匹配node_name, 如果是空格，取消 nodename 匹配
                if !node_name_match_end && is_white_space(current_char) {
                    // 设置节点名匹配完毕
                    node_name_match_end = true;

                    node_name = &xml[start_index..cursor];
                    // node_name 收集完毕，设置开始索引，下面可以开始收集 expression
                    expression_start_index = cursor;
                }

                // 表达式收集必须是在 node_name 收集完之后才有效
                if node_name_match_end {
                    // 是空格，结束收集，并且设置下个开始的 index
                    if is_white_space(current_char) {
                        let expression = &xml[expression_start_index..cursor];
                        // 长度大于0才会被添加
                        if expression.len() > 0 {
                            expressions.push(expression);
                        }
                        // 设置下个开始的index
                        expression_start_index = cursor + 1;
                    }
                }

                // 到标签内第三个字符的时候，需要检测是否是注释 （如果前面的检测都没有动作的话）
                if cursor - start_index == 3 {
                    // 检测到注释节点
                    if xml[start_index..cursor] == ['!', '-', '-'] {
                        // 记录一下注释内容开始的索引
                        let comment_start_index = cursor;

                        while cursor < length {
                            cursor += 1;
                            // 检测最后三位注释结束 （只有超过后三位时才开始检测, 防止注释尾解析到注释头）
                            if cursor >= (comment_start_index + 3)
                                && &xml[(cursor - 3)..cursor] == ['-', '-', '>']
                            {
                                node = Some(Node::Comment(Comment {
                                    value: &xml[comment_start_index..(cursor - 3)],
                                }));
                                is_comment_node = true;
                                break;
                            }
                        }
                        // 因为当前字符是 > 之后的一个字符, 而最后 cursor 还会加1， 所以这里需要 cursor - 1
                        cursor -= 1;

                        // 如果已经是注释，就结束整个标签解析
                        if is_comment_node {
                            break;
                        }
                    }
                }
            }

            // 添加最后的 expression
            let expression = &xml[expression_start_index..cursor];
            // 表达式长度大于0才，会被添加，而且节点名长度也需要大于0，否则不存在表达式，即只是一个没有自身属性的节点
            if expression.len() > 0 && node_name.len() > 0 {
                expressions.push(expression);
            }

            // 如果不是注释节点，则是正常节点，正常处理，注释节点在上面就会被提前设置，故不做动作
            if !is_comment_node {
                // 是否是单标签
                let mut is_single_tag = false;
                // 是否是结束标签
                let mut is_end_tag = false;

                // 这里的node_value指括号内的内容
                let mut node_value = &xml[start_index..cursor];

                // node 主体的长度
                let node_value_len = node_value.len();

                // 如果标签内最后一个字符是 /，则为单标签
                if node_value_len > 1 && node_value[node_value_len - 1] == '/' {
                    // node_value 删除最后一个 /
                    node_value = &node_value[0..node_value_len - 1];
                    // 单标签没有属性，如果 / 之前有一个空格，则空格之后的内容,即 / 会被解析为 expression, 如果只有一个 / 就需要清空
                    if expressions.len() > 0
                        && expressions[0].len() == 1
                        && expressions[0][0] == '/'
                    {
                        // 清空 expressions
                        expressions = Vec::new();
                    }
                    is_single_tag = true;
                }

                // 如果没有发现空白分割，则标签内容就是node_name,没有props
                if node_name.len() <= 0 {
                    // fragment 检测
                    if node_value.len() == 0 {
                        node = Some(Node::Fragment(Fragment { children: None }));
                    } else {
                        // 检测是否是结束标签
                        if node_value[0] == '/' {
                            node_name = &node_value[1..];
                            is_end_tag = true;
                            // 如果仅有一个/则为Fragment
                            if node_value.len() == 1 {
                                node = Some(Node::FragmentEnd(FragmentEnd {}));
                            }
                        } else {
                            node_name = node_value;
                        }
                    }
                }

                // 如果前面的都没有确定 node, 则 node 是 Tag
                if let None = node {
                    // 节点和结束节点检测
                    if is_end_tag {
                        node = Some(Node::TagEnd(TagEnd { node_name }));
                    } else {
                        node = Some(Node::Tag(Tag {
                            node_name,
                            is_single_tag,
                            props: parse_expression_to_props(expressions),
                            children: None,
                        }));
                    }
                }
            }

            if let Some(node) = node {
                // 节点添加到组中
                nodes.push(node);
            }

            // 设置下个字符串开始索引，开始收集字符串
            string_start_index = cursor + 1;
        }
        cursor += 1;
    }
    // 收集末尾的字符串（如果是的话）
    if string_start_index < cursor {
        // 清理两侧空白
        if let Some(string) = trim_string(&xml[string_start_index..cursor]) {
            nodes.push(Node::String(string));
        }
    }
    nodes
}

// 解析("Tag" 类型的 Tag)节点属性，解析完毕的内容会从expressions中被删除，会被添加到到props中
pub fn parse_expression_to_props<'a>(expressions: Expressions<'a>) -> Option<Props<'a>> {
    // 如果长度为0，是空的，就直接返回 None
    if expressions.len() <= 0 as usize {
        return None;
    }
    let mut props: Props = HashMap::new();
    for expression in expressions.iter() {
        // 通过遍历，获得 = 的位置，截取两侧的值，设置为props, 如果没有找到，则设置其为 true
        let mut equal_char_index = 0;
        let len = expression.len();

        while equal_char_index < len {
            if expression[equal_char_index] == '=' {
                // 等于号在最后, 那么设置这个 prop 为true
                if equal_char_index == len {
                    props.insert(
                        &expression[0..equal_char_index],
                        PropValue::BooleanValue(true),
                    );
                    // 结束循环
                    break;
                }
                // 值部分长度2及以上，才需要删除 ' 或 " 这两个字符串包裹符号
                if len >= equal_char_index + 2 {
                    // 值部分第一个字符
                    let first_char = expression[equal_char_index + 1];
                    // 如果首尾相同，且等于 ' 或 ",则判定为是字符串标识符
                    if first_char == expression[len - 1]
                        && (first_char == '\'' || first_char == '"')
                    {
                        // 取字符串中间部分， 去首尾标识符
                        props.insert(
                            &expression[0..equal_char_index],
                            PropValue::StringValue(&expression[equal_char_index + 2..len - 1]),
                        );
                    } else {
                        // 不是字符串标识符，直接截取两侧
                        props.insert(
                            &expression[0..equal_char_index],
                            PropValue::StringValue(&expression[equal_char_index + 1..]),
                        );
                    }
                    // 结束循环
                    break;
                } else {
                    // 值部分长1的话，直接设置值，不需要检测字符串标识符，如果是单个字符串符号，则上面的解析就无法通过
                    props.insert(
                        &expression[0..equal_char_index],
                        PropValue::StringValue(&expression[equal_char_index + 1..]),
                    );
                    // 结束循环
                    break;
                }
            }
            equal_char_index += 1;
        }
        // 没有找到 = ，就直接设置以这个 expression 为 key， 值为 true
        if equal_char_index >= len {
            props.insert(
                &expression[0..equal_char_index],
                PropValue::BooleanValue(true),
            );
        }
    }
    Some(props)
}

use std::vec::IntoIter;

// 将 xml_tree_node 添加到 children 中
fn is_xml_tree_node_add_to_children<'a>(node: Node<'a>, children: &mut Children<'a>) {
    // 类型为 XMLTreeNode 就添加到 children, 不再具有结束标签
    match node {
        Node::Tag(node) => children.push(XMLTreeNode::Tag(node)),
        Node::Fragment(node) => children.push(XMLTreeNode::Fragment(node)),
        Node::Comment(node) => children.push(XMLTreeNode::Comment(node)),
        Node::String(node) => children.push(XMLTreeNode::String(node)),
        _ => (),
    };
}

pub fn parse_to_trees<'a>(nodes: Nodes<'a>) -> Children<'a> {
    let mut parsed_nodes: Children = Vec::new();

    let mut node_iter: IntoIter<Node<'a>> = nodes.into_iter();

    let node_iter_ref = &mut node_iter;

    fn parse<'a>(node_iter: &mut IntoIter<Node<'a>>) -> Option<Node<'a>> {
        let node = node_iter.next();

        match node {
            Some(node) => {
                match node {
                    // 注释节点直接返回
                    Node::Comment(node) => Some(Node::Comment(node)),
                    // 字符串节点直接返回
                    Node::String(node) => Some(Node::String(node)),
                    // 结束标签直接返回
                    Node::FragmentEnd(node) => Some(Node::FragmentEnd(node)),
                    Node::TagEnd(node) => Some(Node::TagEnd(node)),

                    Node::Fragment(mut node) => {
                        let mut children: Children = Vec::new();
                        // 递归解析子节点
                        loop {
                            // 解析不到子元素直接结束掉进程
                            let child = parse(node_iter).unwrap();

                            // 解析直到碎片结束
                            if let Node::FragmentEnd(_) = &child {
                                break;
                            }

                            // 添加子节点到children中
                            is_xml_tree_node_add_to_children(child, &mut children);
                        }
                        node.children = Some(children);
                        Some(Node::Fragment(node))
                    }
                    Node::Tag(mut node) => {
                        // 单节点直接返回
                        if node.is_single_tag {
                            return Some(Node::Tag(node));
                        }
                        let mut children: Children = Vec::new();

                        // 递归解析子节点
                        loop {
                            // 解析不到子元素直接结束掉进程
                            let child = parse(node_iter).unwrap();

                            // 解析直到结束节点
                            if let Node::TagEnd(child) = &child {
                                // 标签名相同，则是结束节点

                                if child.node_name == node.node_name {
                                    break;
                                }
                            }

                            // 添加子节点到children中
                            is_xml_tree_node_add_to_children(child, &mut children);
                        }
                        node.children = Some(children);
                        Some(Node::Tag(node))
                    }
                }
            }
            None => None,
        }
    }
    loop {
        // 循环到迭代器没有内容,并且添加到已解析的nodes中
        match parse(node_iter_ref) {
            Some(node) => {
                // 添加子节点到children中
                is_xml_tree_node_add_to_children(node, &mut parsed_nodes);
            }
            None => break,
        };
    }
    parsed_nodes
}

// 将chars添加到String
fn push_chars(target: &mut String, chars: &[char]) {
    for c in chars.iter() {
        let current_char = *c;
        //   如果换行符就变成 \\r 或者 \\n ，转移其使得符合JSON标准
        if current_char == '\n' {
            target.push_str("\\n");
            continue;
        }
        if current_char == '\r' {
            target.push_str("\\r");
            continue;
        }
        target.push(current_char);
    }
}

pub fn generate_json(xml_tree_nodes: Children, mut result: &mut String) {
    // 没有子节点，直接添加空
    if xml_tree_nodes.len() == 0 {
        result.push_str("[]");
        return;
    }
    result.push('[');
    // 有子节点，遍历
    for node in xml_tree_nodes.into_iter() {
        match node {
            XMLTreeNode::Comment(node) => {
                result.push_str("{\"type\":\"Comment\",\"value\":\"");
                push_chars(&mut result, node.value);
                result.push_str("\"}")
            }
            XMLTreeNode::String(node) => {
                result.push('"');
                push_chars(&mut result, node);
                result.push('"');
            }
            XMLTreeNode::Fragment(node) => {
                result.push_str("{\"type\":\"Fragment\"");
                match node.children {
                    Some(children) => {
                        result.push_str(",\"children\":");
                        generate_json(children, &mut result);
                    }
                    None => (),
                };
                result.push_str("}");
            }
            XMLTreeNode::Tag(node) => {
                result.push_str("{\"type\":\"Tag\",\"nodename\":\"");
                push_chars(&mut result, node.node_name);
                result.push('\"');
                match node.props {
                    Some(props) => {
                        result.push_str(",\"props\":{");
                        for (key, value) in props.into_iter() {
                            result.push('\"');
                            push_chars(&mut result, key);
                            result.push_str("\":");
                            match value {
                                PropValue::BooleanValue(_) => {
                                    result.push_str("true,");
                                }
                                PropValue::StringValue(v) => {
                                    result.push('\"');
                                    push_chars(&mut result, v);
                                    result.push_str("\",");
                                }
                            }
                        }
                        // 最后一个逗号替换为}
                        result.replace_range(result.len() - 1..result.len(), "}");
                    }
                    None => (),
                }
                match node.children {
                    Some(children) => {
                        result.push_str(",\"children\":");
                        generate_json(children, &mut result);
                    }
                    None => (),
                };
                result.push_str("}");
            }
        }
        result.push_str(",");
    }
    // 最后一个逗号替换为]
    result.replace_range(result.len() - 1..result.len(), "]");
}

pub fn xml_compile(raw_xml: &str) -> String {
    // 转化为字符组
    let xml: Vec<char> = raw_xml.chars().collect();
    // 解析为节点
    let nodes = parse_to_nodes(&xml);
    // // 解析节点及其中的属性，生成树形结构
    let trees = parse_to_trees(nodes);

    let mut json = String::from("");
    generate_json(trees, &mut json);
    json
}
