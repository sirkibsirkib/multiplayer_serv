//TODO, use for objects?

struct QuadTree<T> {

}

enum QuadOpt<T> {
    Zero,
    One(T),
    Four(QuadTree<T>),
}
