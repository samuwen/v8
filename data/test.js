function objectFactory() {
    return { foo: "bar", baz: { quz: "hotdog" } };
}

const obj = objectFactory();
const y = obj["baz"].quz;
y;
