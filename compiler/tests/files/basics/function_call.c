int other() {
	return 5;
}

int echo(int value) {
	return value;
}

int main() {
	int initial = other();
	int other_val = echo(5);

	int result = initial - other_val;

	return result;
}
