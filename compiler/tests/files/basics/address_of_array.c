int main() {
	int test[5];
	for (int i = 0; i < 5; i++) {
		test[i] = 10;
	}

	test[3] = 0;

	int* inner = &test[3];

	int result = *inner;
	return result;
}
