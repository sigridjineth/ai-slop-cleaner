# Banned Words

Dynamically loaded banned words and phrases for AI slop detection.
Loaded by the Rust binary at runtime from `rules/banned-words.md`.

## English Buzzwords

| Word | Replacement | Weight |
|------|-------------|--------|
| delve into | explore | 1.0 |
| delve | explore | 1.0 |
| elucidate | explain | 1.0 |
| underscore | stress | 1.0 |
| harness | use | 1.0 |
| leverage | use | 1.0 |
| bolster | strengthen | 1.0 |
| foster | encourage | 1.0 |
| showcase | show | 1.0 |
| streamline | simplify | 1.0 |
| revolutionize | transform | 1.0 |
| unveil | reveal | 1.0 |
| orchestrate | arrange | 1.0 |
| transcend | go beyond | 1.0 |
| exemplify | show | 1.0 |
| augment | expand | 1.0 |
| surpass | exceed | 1.0 |
| pinpoint | identify | 1.0 |
| scrutinize | examine | 1.0 |
| unravel | solve | 1.0 |
| embark | start | 1.0 |
| navigate | handle | 1.0 |
| elevate | raise | 1.0 |
| unlock | open up | 1.0 |
| unleash | release | 1.0 |
| dive | look | 1.0 |
| discover | find | 1.0 |
| craft | make | 1.0 |
| illuminate | clarify | 1.0 |
| pivotal | key | 1.0 |
| meticulous | careful | 1.0 |
| intricate | complex | 1.0 |
| transformative | major | 1.0 |
| groundbreaking | new | 1.0 |
| unparalleled | unmatched | 1.0 |
| comprehensive | thorough | 1.0 |
| robust | strong | 1.0 |
| crucial | vital | 1.0 |
| notable | noteworthy | 1.0 |
| formidable | impressive | 1.0 |
| nuanced | subtle | 1.0 |
| multifaceted | varied | 1.0 |
| paramount | top | 1.0 |
| instrumental | helpful | 1.0 |
| foundational | basic | 1.0 |
| commendable | admirable | 1.0 |
| cutting-edge | latest | 1.0 |
| seamless | smooth | 1.0 |
| vibrant | lively | 1.0 |
| bustling | busy | 1.0 |
| holistic | whole | 1.0 |
| poised | ready | 1.0 |
| remarkable | striking | 1.0 |
| realm | area | 1.0 |
| tapestry | fabric | 1.0 |
| landscape | field | 1.0 |
| beacon | guide | 1.0 |
| hurdles | obstacles | 1.0 |
| testament | proof | 1.0 |
| game-changer | breakthrough | 1.0 |
| journey | process | 1.0 |
| synergy | cooperation | 1.0 |

## English Phrases

| Phrase | Replacement | Weight |
|--------|-------------|--------|
| in today's digital age | | 2.0 |
| it's important to note | | 2.0 |
| furthermore | | 2.0 |
| moreover | | 2.0 |
| not only this, but also this | | 2.0 |
| in conclusion | | 2.0 |
| in closing | | 2.0 |
| let's dive in | | 2.0 |
| let's explore | | 2.0 |
| it's worth noting that | | 2.0 |
| as we navigate | | 2.0 |
| in an era of | | 2.0 |
| it is essential that | make sure | 2.0 |
| it should be noted | | 2.0 |
| in order to | to | 1.5 |
| due to the fact that | because | 1.5 |
| with regard to | about | 1.5 |
| in the event that | if | 1.5 |
| for the purpose of | to | 1.5 |
| at this point in time | now | 1.5 |
| in spite of the fact that | although | 1.5 |
| in the absence of | without | 1.5 |
| it is evident that | clearly | 1.5 |
| it is apparent that | clearly | 1.5 |
| there is a need for | we need | 1.5 |
| a significant number of | many | 1.5 |
| a considerable amount of | much | 1.5 |
| it is interesting to note that | | 2.0 |
| one could argue that | | 2.0 |
| it is possible that | | 2.0 |
| there is a possibility that | | 2.0 |
| it may be worth considering | | 2.0 |
| arguably | | 2.0 |
| potentially | | 2.0 |
| in some cases | | 2.0 |
| to a certain extent | | 2.0 |
| by and large | | 2.0 |
| pipeline | flow | 1.0 |
| framework | system | 1.0 |
| scalable | able to grow | 1.0 |
| insight | finding | 1.0 |
| impact | effect | 1.0 |

## Korean Buzzwords

| Word | Replacement | Weight |
|------|-------------|--------|
| 결론적으로 | | 2.0 |
| 요약하면 | | 2.0 |
| 종합하면 | | 2.0 |
| 정리하자면 | | 2.0 |
| 시사하는 바가 크다 | 의미가 있다 | 1.5 |
| 주목할 만하다 | 눈에 뜨인다 | 1.5 |
| 간과할 수 없다 | 놓치면 안 된다 | 1.5 |
| 무시할 수 없다 | 작지 않다 | 1.5 |
| 지평을 연다 | 길을 연다 | 1.5 |
| 방점을 찍는다 | 강조한다 | 1.5 |
| 의미가 적지 않다 | 의미가 있다 | 1.5 |
| 의미심장하다 | 의미가 있다 | 1.5 |
| 혁신적인 | 새롭다 | 1.5 |
| 획기적인 | 새롭다 | 1.5 |
| 전례 없는 | 드문 | 1.5 |
| 압도적 | 큰 | 1.5 |
| 막강한 | 강한 | 1.5 |
| 폭발적 | 빠른 | 1.5 |
| 파격적 | 이례적인 | 1.5 |
| 대대적 | 큰 규모의 | 1.5 |
| 강력한 | 강한 | 1.5 |
| 치열한 | 거센 | 1.5 |
| 뜨거운 | 활발한 | 1.5 |
| 가능성을 열어준다 | 가능하게 한다 | 1.5 |
| 새로운 장을 열다 | 새 단계로 넘어가다 | 1.5 |
| 시대가 도래했다 | 시대가 왔다 | 1.5 |
| 매우 | | 1.0 |
| 정말 | | 1.0 |
| 진짜로 | | 1.0 |
| 대단히 | | 1.0 |
| 극히 | | 1.0 |
