#include <iostream>
#include <fstream>
#include <string>
#include <vector>
using namespace std;

const int header_size = 1024 * 1024;
const int page_size = 1024 * 4;

inline unsigned align_val(unsigned val) {
    unsigned o = page_size - 1;
    return (val + o) & ~o;
}

struct File {
    ssize_t offset;
    ssize_t size;
    string name;
};

int main(int argc, char** argv) {
    cout << "Making simple filesystem..." << endl;
    cout << "Using " << argv[1] << " as target image" << endl;
    ofstream hdd(argv[1], ios::binary | ios::out);
    vector<File> files;
    ssize_t cum_sz = header_size;
    for (int i = 2; i < argc; i++) {
        string filename(argv[i]);
        auto pos = filename.find_last_of('/');
        string fsname = filename.substr(pos, filename.size() - pos);
        cout << "Processing " << filename << " (" << fsname << " in fs)" << endl;
        ifstream file(filename, ios::binary | ios::in);
        file.seekg(0, ios::end);
        auto sz = file.tellg();
        files.push_back({ cum_sz, sz, fsname });
        file.seekg(sz);
        hdd.seekp(cum_sz);
        cout << "Write file of size " << sz << " to pos " << cum_sz << endl;
        cum_sz += align_val(sz);
        file.seekg(0);
        for (int i = 0; i < sz; i++) hdd.put(file.get());
        for (int i = sz; i < align_val(sz); i++) hdd.put(0);
    }
    cout << "Writing header..." << endl;
    cout << "Little endian, ssize_t=" << sizeof(ssize_t) << endl;
    for (int i = 0; i < files.size(); i++) {
        auto &p = files[i];
        hdd.seekp(i * 1024);
        ssize_t sz = p.size;
        ssize_t offset = p.offset;
        hdd.write((char*) &sz, sizeof(ssize_t));
        hdd.write((char*) &offset, sizeof(ssize_t));
        hdd.write(p.name.c_str(), p.name.length() + 1);
    }
    cout << files.size() << " files written." << endl;
    return 0;
}
