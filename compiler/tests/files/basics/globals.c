
typedef struct {
	int crn;
	char name[10];
	int hours;
} course;

typedef struct {
	int id;
	char name[50];
	course courses[4];
} student;

student *students = (student*) 0;
int studentCount = 0;

int getStudentIndex(int id) {
	for(int i = 0; i < studentCount; i++) {
		if (students[i].id == id) {
			return i;
		}
	}

	return -1;
}

int main() {
	return 0;
}
