pub mod frame;
pub mod station;

#[cfg(test)]
mod tests {
    use super::station::*;
    #[test]
    fn test_station() -> Result<(), serde_yaml::Error> {
        //注意 解析用户输入时需加确认，以免崩溃
        //選用 yaml 的原因：雖 json 足堪此任，但由於下列諸因素考量，
        // 1. json 易閲性顯然不比 yaml，yaml 更为瞭然
        // 2. json 使用引號等分界，甚繁冗，不易編輯
        // 從用戶的角度考量，yaml更爲合宜。
        let json = r#"
        title: test_station
        author: admin
        nodes: 
          - id: 1
            track_id: X3JG
            adjoint_nodes: [3, 7]
            conflicted_nodes: [4, 5]
            line: [[0.1, 0.7], [1.1, 5.6]]
            joint: [true, true]

        signals: 
          - id: X
            pos: [2.0, 3.0]
            dir: right
            sig_type: home_signal
            sig_mnt: ground_mounting
            btn_id: LA
            protect_node_id: 12
        "#;

        println!("{:#?}", serde_yaml::from_str::<NewStationData>(json)?);

        Ok(())
    }
}
