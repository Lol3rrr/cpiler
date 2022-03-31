int main() {
	int result = 10;

	int* ptr = &result;
	*ptr = 0;

	return result;
}
