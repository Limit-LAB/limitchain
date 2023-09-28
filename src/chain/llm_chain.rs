use std::collections::BTreeMap;

use async_trait::async_trait;
use itertools::Itertools;
use serde::Serialize;

use crate::{
    prompt_template::PromptTemplate,
    schema::{Generation, Message},
};

use super::{Chain, LLM};

/// {question} -> {answer}
#[derive(Debug, Clone, Serialize)]
pub struct LLMChain<Executor: Send + Sync + Serialize + LLM + Clone> {
    history: Vec<Message>,
    prompt_template: Option<PromptTemplate>,
    executor: Executor,
}

impl<Executor: Send + Sync + Serialize + LLM + Clone> LLMChain<Executor> {
    pub fn new(
        history: Vec<Message>,
        prompt_template: Option<PromptTemplate>,
        executor: Executor,
    ) -> Self {
        Self {
            history,
            prompt_template,
            executor,
        }
    }
}

#[async_trait]
impl<Executor: Send + Sync + Serialize + LLM + Clone> Chain for LLMChain<Executor> {
    async fn generate(
        &self,
        input: &BTreeMap<String, String>,
        stop: Vec<String>,
    ) -> Option<Generation> {
        let prompt = self.prepare_prompt(input);
        let llm = self.get_llm();
        let mut his = self.history.clone();
        his.push(prompt?);
        Some(llm.generate(his, stop).await)
    }

    fn get_input_keys(&self) -> Vec<String> {
        self.prompt_template
            .as_ref()
            .map_or(vec!["question".to_string()], |t| {
                t.variables.keys().cloned().collect_vec()
            })
    }

    fn get_output_keys(&self) -> Vec<String> {
        vec!["answer".to_string()]
    }

    fn get_prompt_template(&self) -> PromptTemplate {
        self.prompt_template
            .clone()
            .unwrap_or_else(|| PromptTemplate::from("{question}".to_string()))
    }

    fn get_llm(&self) -> impl LLM {
        self.executor.clone()
    }

    fn create_output(&self, generation: Generation) -> Option<BTreeMap<String, Message>> {
        let mut output = BTreeMap::new();
        output.insert("answer".to_string(), generation.text[0].clone());
        Some(output)
    }
}

#[tokio::test]
async fn test_llm_chain_openai() {
    use crate::btreemap;
    use crate::client::openai::*;
    dotenvy::dotenv().unwrap();

    let chain = LLMChain {
        history: vec![],
        prompt_template: Some(PromptTemplate::from("{question}".to_string())),
        executor: OpenAIClient::default(),
    };

    let res = chain
        .apply(
            &btreemap! {
                "question".to_string() => "What is human?".to_string()
            },
            vec!["stop".to_string()],
        )
        .await;

    println!("{:?}", res);
}

#[tokio::test]
async fn test_llm_chain_glm() {
    use crate::btreemap;
    use crate::client::glm::*;
    dotenvy::dotenv().unwrap();

    let chain = LLMChain {
        history: vec![
            Message {
                role: "user".to_string(),
                content: "你打的真菜".to_string(),
            },
            Message {
                role: "bot".to_string(),
                content: "上路被三人越塔，打野不在我怎么去，你告诉我？上路被三人越塔我都能保得住他吗？如果盲僧在的话我为什么不在？你告诉我，昂？盲僧都没有在为什么我要去......为...盲僧都不在你告诉我为什么我要去啊？啊？他被打野先越塔然后中单赶过去了，盲僧不在我为什么要去啊？啊？你...你告诉我，来，盲僧不在我为什么要去？你...来，我给你房管，你给我说话，来，这个叫你mud bee尊尼获加的这个臭.寄.吧.杠精你给我说话，来，你今天要说不明白你m明天就被车创死。你懂不懂？你m，我就看不惯你这种低分g在这抬杠呢。打野都没有反蹲到上路我怎么...我怎么保他？啊？c.n.m打野不在上路我怎么保他？不是急眼了你能说明白你就行，行不行？不...g东西你什么都说不明白你在这穷抬杠有什么意义吗？你告诉我？
                你白银觉得是我的锅，那就是我的锅，为什么你知道吗？因为白银说的话，就像是一个癌症晚期患者说的话一样。他都已经这样了，你为什么不顺从他呢?你总要给人最后一段时间一个好的回忆吧，最后的时光里。因为白银这个段位很尴尬，白银黄金再往上一点，白金钻石，可能说，欸，有点实力，能操作一下。白银往下，黄铜，一到五，啊，人家是纯属玩游戏的，因为太垃 圾了，自己也知道自己没什么实力。但白银，上不去下不来的这个段位，他觉得，黄铜的人不配跟他一起玩儿，对吧？黄铜是最垃 圾的。但是呢他想上去，他又上不去，所以这个分段是最尴尬的，没办法，卡在这里了。想操作，又操作不起来，掉下去吧，他又觉得不值得，对吧，我好不容易从黄铜打到...打到白银了，我为什么还要掉下去呢?这个人说优越g 越说越起劲，为什么他会这么说?因为他是白银呐。他觉得你比我段位高，你说的任何话都是优越，我并不管你说的有没有道理。我白银，我最猛，我S8我上我能夺冠，那打比赛全是s.b。你比我段位高你说话就是放屁，这就是这种人的想法。但是呢，他的想法是对的，为什么呢？因为他癌症晚期。没办法，我同意，对不起，我优越了。可能是我膨胀了，不好意思啊，我膨胀了。我白银是没操作，难道我就看不懂谁背锅吗？不是，如果你看得懂的话，就不会在这里抬杠了，对吧。
                ".to_string(),
            },
        ],
        prompt_template: Some(PromptTemplate::from("{question}".to_string())),
        executor: GLMClient {
            model: "characterglm".to_string(),
            meta: Some(GLMCharacterMeta {
                user_info: "2B青年，喜欢玩英雄联盟".to_string(),
                bot_info: "这哈比下的米诺，真是欧西给几遍也哇袄不够的，愿称其为一种冷峻的奥利安费。看似欧内的手淡淡地好汗，偶有哈姆的哈贝贝穿插其间，文宇背后的一坨史却足以哈比下。配合上几乎不加额外修饰的么么哒米诺，这部视频便不再是普通的一坨史，更像是一部微缩的啊嘿露西，一场时长极短的allin，似乎有些我超冰，匆匆而过转眼就尊尼获加，这又何其像是“说的道理”。".to_string(),
                bot_name: "电棍".to_string(),
                user_name: "akarachan".to_string(),
            }),
            temperature: Some(0.8),
            ..Default::default()
        },
    };

    let res = chain
        .apply(
            &btreemap! {
                "question".to_string() => "
                你他妈三级被抓三次，
                阿卡丽打的菜的一，
                一开局0/3/0，
                一辈子卡钻2，
                你就白银局炸炸鱼，你拿什么打职业，
                我弟青铜打的都比你6,
                刀也不补，团也不打，晃晃晃
                然后被单杀".to_string()
            },
            vec!["stop".to_string()],
        )
        .await;

    println!("{:?}", res);
}
