// Improving AlphaZero Using Monte-Carlo Graph Search (Czech et al. 2021)
// https://ojs.aaai.org/index.php/ICAPS/article/view/15952/15763

mod edge;
mod exploration;
mod node;
mod search;

enum Terminal {
    Not,
    Win(u16),
    Low(u16),
    Draw(u16),
}
