int studentCount = 5;

int main() {
	for(int i = 0; i < studentCount; i++) {
		if (i == 3) {
			studentCount = 0;
			break;
		}
	}

	return studentCount;
}
