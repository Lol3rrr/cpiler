int main() {
	int content[10];
	content[0] = 9;

	for (int i = 1; i < 10; i++) {
		content[i] = 1;
	}

	int result = content[0];
	for (int j = 1; j < 10; j++) {
		result -= content[j];
	}

	return result;
}
