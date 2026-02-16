/// 将给定的 `Vec` 按 `page_size` 分页，返回一个包含各页的 `Vec<Vec<T>>`。
///
/// # 参数
/// * `vec` - 原始向量，会被消耗（所有权移入函数）
/// * `page_size` - 每页的元素数量，必须大于 0；若为 0 则返回空向量
///
/// # 示例
/// ```
/// let v = vec![1, 2, 3, 4, 5];
/// let pages = list_pagination(v, 2);
/// assert_eq!(pages, vec![vec![1, 2], vec![3, 4], vec![5]]);
/// ```
pub fn list_pagination<T>(mut vec: Vec<T>, page_size: usize) -> Vec<Vec<T>> {
    if page_size == 0 {
        // 分页大小为零时无法分页，返回空结果
        return Vec::new();
    }

    let mut result = Vec::new();
    while !vec.is_empty() {
        let remaining = vec.len();
        let take = remaining.min(page_size); // 实际可取的元素数
        let chunk: Vec<T> = vec.drain(..take).collect(); // 移动所有权，无需克隆
        result.push(chunk);
    }
    result
}