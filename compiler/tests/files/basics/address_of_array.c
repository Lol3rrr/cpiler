int main() {
	int test[3];
	
	test[0] = 10;
	test[1] = 10;
	test[2] = 10;

	test[1] = 0;

	int* inner = &test[1];

	int result = *inner;
	return result;
}
